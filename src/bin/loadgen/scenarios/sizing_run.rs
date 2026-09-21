//! One-command sizing methodology: geometric connection, throughput, and
//! memory sweeps against one self-contained server child process. The output
//! records both the measured result and every parameter needed to reproduce it.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use std::mem::MaybeUninit;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::raw::c_int;
#[cfg(target_os = "linux")]
use std::os::raw::c_long;

use anyhow::{bail, ensure, Context, Result};
use serde::Serialize;
use serde_json::json;
use tokio::sync::{watch, Semaphore};
use tokio::task::{JoinHandle, JoinSet};
use wss_mux::envelope::ServerFrame;

use crate::cli::{Cli, SizingArgs};
use crate::client::{self, Recv, Ws};
use crate::metrics;
use crate::report::Percentiles;
use crate::scenarios::post_event;
use crate::server::{self, Target};
use crate::token;

const LATENCY_CLIFF_MULTIPLIER: f64 = 2.0;
const LATENCY_CLIFF_MIN_INCREASE_MS: f64 = 20.0;
const TARGET_RATE_FRACTION: f64 = 0.90;
const REQUEST_FRACTION: f64 = 0.70;
const LIMIT_MULTIPLIER: f64 = 1.50;
const MAX_LATENCY_SAMPLES: usize = 200_000;
const PRODUCER_WORKERS: usize = 16;
const DELIVERY_SETTLE_SECS: u64 = 2;
const CHILD_SIGNING_KEY_ENV: &str = "WSS_MUX_LOADGEN_CHILD_SIGNING_KEY";
const CHILD_PUSH_TOKEN_ENV: &str = "WSS_MUX_LOADGEN_CHILD_PUSH_TOKEN";

#[derive(Debug, Serialize)]
pub struct SizingReport {
    pub instance_label: String,
    pub version: String,
    pub commit: String,
    pub dirty: bool,
    pub build_profile: String,
    pub provenance_source: String,
    pub date: String,
    pub methodology: Methodology,
    pub runs: Vec<SizingRunReport>,
}

#[derive(Debug, Serialize)]
pub struct Methodology {
    pub stream: String,
    pub resource_process: String,
    pub ramp_multiplier: u8,
    pub step_duration_secs: u64,
    pub connection_start: usize,
    pub connection_max: usize,
    pub subscriptions_per_connection: usize,
    pub connection_event_rate_per_sec: u64,
    pub fixed_connections: usize,
    pub event_rate_start_per_sec: u64,
    pub event_rate_max_per_sec: u64,
    pub slow_consumer_every: usize,
    pub memory_payload_bytes: usize,
    pub available_cpus: usize,
    pub producer_concurrency: usize,
    pub latency_sample_limit: usize,
    pub delivery_settle_secs: u64,
    pub cpu_saturation_percent: u16,
    pub latency_cliff_multiplier: f64,
    pub latency_cliff_min_increase_ms: f64,
    pub target_rate_fraction: f64,
    pub request_fraction: f64,
    pub limit_multiplier: f64,
    pub saturation_criteria: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SizingRunReport {
    pub name: String,
    pub saturation_point: u64,
    pub saturation_unit: String,
    pub saturated: bool,
    pub signals: Vec<String>,
    pub p99_latency_ms: f64,
    pub achieved_event_rate_per_sec: f64,
    pub events_dropped: u64,
    pub rss: String,
    pub cpu: String,
    pub recommended_requests: Resources,
    pub recommended_limits: Resources,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Resources {
    pub cpu: String,
    pub memory: String,
}

#[derive(Clone)]
struct DeliveryProbe {
    origin: Instant,
    current_step: Arc<AtomicU64>,
    next_step: Arc<AtomicU64>,
    samples: Arc<Mutex<Vec<f64>>>,
}

impl DeliveryProbe {
    fn new() -> Self {
        Self {
            origin: Instant::now(),
            current_step: Arc::new(AtomicU64::new(0)),
            next_step: Arc::new(AtomicU64::new(1)),
            samples: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn begin_step(&self) -> u64 {
        let step = self.next_step.fetch_add(1, Ordering::Relaxed);
        self.current_step.store(step, Ordering::Release);
        self.samples.lock().expect("latency probe lock").clear();
        step
    }

    fn record(&self, frame: &ServerFrame) {
        let ServerFrame::Event { payload, .. } = frame else {
            return;
        };
        let Some(stamp) = payload.get("t_nanos").and_then(|value| value.as_u64()) else {
            return;
        };
        let Some(step) = payload.get("sizing_step").and_then(|value| value.as_u64()) else {
            return;
        };
        if step != self.current_step.load(Ordering::Acquire) {
            return;
        }
        let mut samples = self.samples.lock().expect("latency probe lock");
        if samples.len() < MAX_LATENCY_SAMPLES {
            let now = self.origin.elapsed().as_nanos() as u64;
            samples.push(now.saturating_sub(stamp) as f64 / 1_000_000.0);
        }
    }

    fn p99(&self) -> f64 {
        let samples = std::mem::take(&mut *self.samples.lock().expect("latency probe lock"));
        Percentiles::from_millis(samples).p99
    }

    fn sample_count(&self) -> usize {
        self.samples.lock().expect("latency probe lock").len()
    }
}

struct Subscribers {
    drains: Vec<JoinHandle<()>>,
    slow: Vec<Ws>,
    attempted: usize,
    connected: usize,
    probe_assigned: bool,
}

impl Subscribers {
    fn new() -> Self {
        Self {
            drains: Vec::new(),
            slow: Vec::new(),
            attempted: 0,
            connected: 0,
            probe_assigned: false,
        }
    }

    async fn add_to(
        &mut self,
        cli: &Cli,
        target: &Target,
        desired: usize,
        slow_every: Option<usize>,
        probe: &DeliveryProbe,
    ) -> Result<()> {
        let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;
        let ws_base = target.instance_ws(target.subscriber_index()).to_string();

        while self.attempted < desired {
            let chunk_end = desired.min(self.attempted.saturating_add(256));
            let mut opens = JoinSet::new();
            for index in self.attempted..chunk_end {
                let ws_base = ws_base.clone();
                let token = token.clone();
                let stream = cli.stream.clone();
                opens.spawn(async move {
                    let mut ws = client::connect(&ws_base).await?;
                    client::auth_and_subscribe(
                        &mut ws,
                        &token,
                        &format!("s{index}"),
                        &stream,
                        None,
                    )
                    .await?;
                    Ok::<_, anyhow::Error>((index, ws))
                });
            }
            self.attempted = chunk_end;

            while let Some(opened) = opens.join_next().await {
                let Ok(Ok((index, mut ws))) = opened else {
                    continue;
                };
                self.connected += 1;
                let is_slow = slow_every
                    .map(|every| (index + 1) % every == 0)
                    .unwrap_or(false);
                if is_slow {
                    self.slow.push(ws);
                    continue;
                }

                let record_latency = !self.probe_assigned;
                self.probe_assigned |= record_latency;
                let probe = probe.clone();
                self.drains.push(tokio::spawn(async move {
                    loop {
                        match client::next(&mut ws, Duration::from_millis(500)).await {
                            Ok(Recv::Event(frame)) if record_latency => probe.record(&frame),
                            Ok(Recv::Closed) | Err(_) => break,
                            Ok(_) => {}
                        }
                    }
                }));
            }
        }
        Ok(())
    }

    async fn shutdown(mut self) {
        self.slow.clear();
        for drain in &self.drains {
            drain.abort();
        }
        for drain in self.drains.drain(..) {
            let _ = drain.await;
        }
    }
}

impl Drop for Subscribers {
    fn drop(&mut self) {
        for drain in &self.drains {
            drain.abort();
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Usage {
    peak_cpu_percent: f64,
    peak_rss_kib: u64,
}

#[derive(Debug)]
struct LoadResult {
    achieved_rate: f64,
    failed: u64,
    p99_latency_ms: f64,
    events_dropped: u64,
}

#[derive(Debug)]
struct Observation {
    point: u64,
    intended_rate: u64,
    target_connections: usize,
    active_connections: usize,
    load: LoadResult,
    usage: Usage,
}

struct SizingServerProcess {
    child: Child,
    _parent_guard: ChildStdin,
    ready_file: PathBuf,
}

impl SizingServerProcess {
    async fn start(cli: &Cli) -> Result<(Self, Target)> {
        let executable = std::env::current_exe().context("locate wss-mux-loadgen executable")?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before Unix epoch")?
            .as_nanos();
        let ready_file = std::env::temp_dir().join(format!(
            "wss-mux-sizing-server-{}-{nonce}.ready",
            std::process::id()
        ));
        let mut child = Command::new(executable)
            .args(["sizing-server", "--ready-file"])
            .arg(&ready_file)
            .env(CHILD_SIGNING_KEY_ENV, &cli.signing_key)
            .env(CHILD_PUSH_TOKEN_ENV, &cli.push_token)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .context("start isolated sizing server process")?;
        let parent_guard = child
            .stdin
            .take()
            .context("open isolated sizing server parent guard")?;
        let mut process = Self {
            child,
            _parent_guard: parent_guard,
            ready_file,
        };

        let deadline = Instant::now() + Duration::from_secs(10);
        let base_url = loop {
            if let Some(status) = process
                .child
                .try_wait()
                .context("check isolated sizing server")?
            {
                bail!("isolated sizing server exited before readiness: {status}");
            }
            if let Ok(value) = fs::read_to_string(&process.ready_file) {
                let value = value.trim();
                if value.starts_with("http://") {
                    break value.to_string();
                }
            }
            ensure!(
                Instant::now() < deadline,
                "isolated sizing server did not become ready within 10 seconds"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        };
        let _ = fs::remove_file(&process.ready_file);
        let target = server::start(Some(base_url), 0, &cli.signing_key, &cli.push_token).await?;
        Ok((process, target))
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for SizingServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.ready_file);
    }
}

pub async fn run(cli: &Cli, args: &SizingArgs) -> Result<SizingReport> {
    validate(cli, args)?;
    let (commit, dirty) = current_provenance()?;
    let available_cpus = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let (server_process, target) = SizingServerProcess::start(cli).await?;
    let server_pid = server_process.pid();

    let connections = connection_sweep(cli, args, &target, available_cpus, server_pid).await?;
    wait_for_connections(&target, 0).await?;
    let throughput =
        event_rate_sweep(cli, args, &target, available_cpus, server_pid, false).await?;
    wait_for_connections(&target, 0).await?;
    let memory = event_rate_sweep(cli, args, &target, available_cpus, server_pid, true).await?;

    Ok(SizingReport {
        instance_label: args.instance_label.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit,
        dirty,
        build_profile: if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        },
        provenance_source: "runtime_checkout".to_string(),
        date: iso8601_utc(SystemTime::now())?,
        methodology: Methodology {
            stream: cli.stream.clone(),
            resource_process: "isolated_server".to_string(),
            ramp_multiplier: 2,
            step_duration_secs: cli.duration,
            connection_start: args.connection_start,
            connection_max: args.connection_max,
            subscriptions_per_connection: 1,
            connection_event_rate_per_sec: args.connection_event_rate,
            fixed_connections: args.fixed_connections,
            event_rate_start_per_sec: args.event_rate_start,
            event_rate_max_per_sec: args.event_rate_max,
            slow_consumer_every: args.slow_every,
            memory_payload_bytes: args.memory_payload_bytes,
            available_cpus,
            producer_concurrency: PRODUCER_WORKERS,
            latency_sample_limit: MAX_LATENCY_SAMPLES,
            delivery_settle_secs: DELIVERY_SETTLE_SECS,
            cpu_saturation_percent: args.cpu_threshold_percent,
            latency_cliff_multiplier: LATENCY_CLIFF_MULTIPLIER,
            latency_cliff_min_increase_ms: LATENCY_CLIFF_MIN_INCREASE_MS,
            target_rate_fraction: TARGET_RATE_FRACTION,
            request_fraction: REQUEST_FRACTION,
            limit_multiplier: LIMIT_MULTIPLIER,
            saturation_criteria: [
                "load_generator_exhausted",
                "events_dropped",
                "cpu_pegged",
                "p99_latency_cliff",
                "target_rate_missed",
                "push_failures",
            ]
            .map(str::to_string)
            .to_vec(),
        },
        runs: vec![connections, throughput, memory],
    })
}

pub async fn serve(_cli: &Cli, ready_file: &Path) -> Result<()> {
    let signing_key = std::env::var(CHILD_SIGNING_KEY_ENV)
        .with_context(|| format!("read {CHILD_SIGNING_KEY_ENV}"))?;
    let push_token = std::env::var(CHILD_PUSH_TOKEN_ENV)
        .with_context(|| format!("read {CHILD_PUSH_TOKEN_ENV}"))?;
    let target = server::start(None, 0, &signing_key, &push_token).await?;
    std::thread::spawn(|| {
        let mut input = std::io::stdin();
        let mut byte = [0u8; 1];
        loop {
            match input.read(&mut byte) {
                Ok(0) | Err(_) => std::process::exit(0),
                Ok(_) => {}
            }
        }
    });
    fs::write(ready_file, target.producer_base())
        .with_context(|| format!("write sizing server readiness at {}", ready_file.display()))?;
    std::future::pending::<()>().await;
    Ok(())
}

pub fn write(path: &Path, report: &SizingReport) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create sizing output directory {}", parent.display()))?;
    }
    let file_name = path
        .file_name()
        .context("sizing output must name a file")?
        .to_string_lossy();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_nanos();
    let temporary =
        path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .with_context(|| format!("create temporary sizing output {}", temporary.display()))?;
        serde_json::to_writer_pretty(&mut file, report).context("serialize sizing result")?;
        file.write_all(b"\n").context("finish sizing result")?;
        file.sync_all().context("sync sizing result")?;
        fs::rename(&temporary, path)
            .with_context(|| format!("publish sizing result at {}", path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn validate(cli: &Cli, args: &SizingArgs) -> Result<()> {
    ensure!(
        cli.target.is_none(),
        "sizing-run is self-contained and does not accept --target"
    );
    ensure!(
        cli.peers == 0,
        "sizing-run measures one instance and does not accept --peers"
    );
    ensure!(
        cli.stream == "loadgen",
        "sizing-run requires the standard loadgen stream"
    );
    ensure!(cli.duration > 0, "--duration must be greater than zero");
    ensure!(
        !args.instance_label.trim().is_empty(),
        "--instance-label must not be empty"
    );
    ensure!(
        args.connection_start > 0,
        "--connection-start must be greater than zero"
    );
    ensure!(
        args.connection_start <= args.connection_max,
        "--connection-start must not exceed --connection-max"
    );
    ensure!(
        args.connection_event_rate > 0,
        "--connection-event-rate must be greater than zero"
    );
    ensure!(
        args.fixed_connections > 0,
        "--fixed-connections must be greater than zero"
    );
    ensure!(
        args.event_rate_start > 0,
        "--event-rate-start must be greater than zero"
    );
    ensure!(
        args.event_rate_start <= args.event_rate_max,
        "--event-rate-start must not exceed --event-rate-max"
    );
    ensure!(
        args.slow_every >= 2,
        "--slow-every must be at least two so the memory sweep retains a latency probe"
    );
    ensure!(
        args.slow_every <= args.fixed_connections,
        "--slow-every must not exceed --fixed-connections"
    );
    ensure!(
        args.memory_payload_bytes > 0,
        "--memory-payload-bytes must be greater than zero"
    );
    Ok(())
}

async fn connection_sweep(
    cli: &Cli,
    args: &SizingArgs,
    target: &Target,
    available_cpus: usize,
    server_pid: u32,
) -> Result<SizingRunReport> {
    let probe = DeliveryProbe::new();
    let mut subscribers = Subscribers::new();
    let mut baseline_p99 = None;
    let mut final_report = None;

    for point in geometric_steps(args.connection_start as u64, args.connection_max as u64) {
        subscribers
            .add_to(cli, target, point as usize, None, &probe)
            .await?;
        let active = wait_for_connections(target, subscribers.connected).await?;
        let (load, usage) = measure_step(
            cli,
            target,
            args.connection_event_rate,
            64,
            &probe,
            server_pid,
        )
        .await?;
        let observation = Observation {
            point,
            intended_rate: args.connection_event_rate,
            target_connections: point as usize,
            active_connections: active,
            load,
            usage,
        };
        let signals = saturation_signals(
            &observation,
            baseline_p99,
            args.cpu_threshold_percent,
            available_cpus,
        );
        if signals.is_empty() && baseline_p99.is_none() && observation.load.p99_latency_ms > 0.0 {
            baseline_p99 = Some(observation.load.p99_latency_ms);
        }
        let terminal = !signals.is_empty();
        let saturated = signals
            .iter()
            .any(|signal| signal != "load_generator_exhausted");
        final_report = Some(to_report(
            "connections",
            "connections",
            &observation,
            saturated,
            signals,
        ));
        if terminal {
            break;
        }
    }

    subscribers.shutdown().await;
    let mut report = final_report.context("connection sweep produced no observations")?;
    if report.signals.is_empty() {
        report.signals = vec!["sweep_limit".to_string()];
    }
    Ok(report)
}

async fn event_rate_sweep(
    cli: &Cli,
    args: &SizingArgs,
    target: &Target,
    available_cpus: usize,
    server_pid: u32,
    memory: bool,
) -> Result<SizingRunReport> {
    let probe = DeliveryProbe::new();
    let mut subscribers = Subscribers::new();
    subscribers
        .add_to(
            cli,
            target,
            args.fixed_connections,
            memory.then_some(args.slow_every),
            &probe,
        )
        .await?;
    let active = wait_for_connections(target, subscribers.connected).await?;
    let mut baseline_p99 = None;
    let mut final_report = None;

    for point in geometric_steps(args.event_rate_start, args.event_rate_max) {
        let payload_bytes = if memory {
            args.memory_payload_bytes
        } else {
            64
        };
        let (load, usage) =
            measure_step(cli, target, point, payload_bytes, &probe, server_pid).await?;
        let observation = Observation {
            point,
            intended_rate: point,
            target_connections: args.fixed_connections,
            active_connections: active,
            load,
            usage,
        };
        let signals = saturation_signals(
            &observation,
            baseline_p99,
            args.cpu_threshold_percent,
            available_cpus,
        );
        if signals.is_empty() && baseline_p99.is_none() && observation.load.p99_latency_ms > 0.0 {
            baseline_p99 = Some(observation.load.p99_latency_ms);
        }
        let terminal = !signals.is_empty();
        let saturated = signals
            .iter()
            .any(|signal| signal != "load_generator_exhausted");
        final_report = Some(to_report(
            if memory { "memory" } else { "throughput" },
            "events_per_second",
            &observation,
            saturated,
            signals,
        ));
        if terminal {
            break;
        }
    }

    subscribers.shutdown().await;
    let mut report = final_report.context("event-rate sweep produced no observations")?;
    if report.signals.is_empty() {
        report.signals = vec!["sweep_limit".to_string()];
    }
    Ok(report)
}

async fn measure_step(
    cli: &Cli,
    target: &Target,
    event_rate: u64,
    payload_bytes: usize,
    probe: &DeliveryProbe,
    server_pid: u32,
) -> Result<(LoadResult, Usage)> {
    let sizing_step = probe.begin_step();
    let http = reqwest::Client::new();
    let base = target.producer_base().to_string();
    let before = metrics::scrape(&http, &base).await?;
    let (stop_tx, stop_rx) = watch::channel(false);
    let sampler = tokio::spawn(sample_usage(server_pid, stop_rx));
    let started = Instant::now();
    let deadline = started + Duration::from_secs(cli.duration);
    let period = Duration::from_secs_f64(1.0 / event_rate as f64).max(Duration::from_nanos(1));
    let padding = "x".repeat(payload_bytes.saturating_sub(64));
    let mut producers = JoinSet::new();
    let permits = Arc::new(Semaphore::new(PRODUCER_WORKERS));
    let mut ticker = tokio::time::interval(period);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut ok = 0u64;
    let mut failed = 0u64;

    loop {
        ticker.tick().await;
        if Instant::now() >= deadline {
            break;
        }
        while let Some(result) = producers.try_join_next() {
            let (request_ok, request_failed) = result.context("sizing producer task panicked")?;
            ok += request_ok;
            failed += request_failed;
        }
        let Ok(permit) = Arc::clone(&permits).try_acquire_owned() else {
            continue;
        };
        let http = http.clone();
        let base = base.clone();
        let push_token = cli.push_token.clone();
        let stream = cli.stream.clone();
        let padding = padding.clone();
        let origin = probe.origin;
        producers.spawn(async move {
            let _permit = permit;
            let stamp = origin.elapsed().as_nanos() as u64;
            let body = serde_json::to_vec(&json!({
                "stream": stream,
                "payload": {
                    "t_nanos": stamp,
                    "sizing_step": sizing_step,
                    "padding": padding.as_str(),
                },
            }))
            .expect("serialize sizing event");
            match post_event(&http, &base, &push_token, &body).await {
                Ok(status) if status.is_success() => (1, 0),
                _ => (0, 1),
            }
        });
    }

    while let Some(result) = producers.join_next().await {
        let (worker_ok, worker_failed) = result.context("sizing producer task panicked")?;
        ok += worker_ok;
        failed += worker_failed;
    }
    let expected_samples = (ok as usize).min(MAX_LATENCY_SAMPLES);
    let settle_deadline = Instant::now() + Duration::from_secs(DELIVERY_SETTLE_SECS);
    while probe.sample_count() < expected_samples && Instant::now() < settle_deadline {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let elapsed = cli.duration as f64;
    let after = metrics::scrape(&http, &base).await?;
    let _ = stop_tx.send(true);
    let usage = sampler.await.context("resource sampler task panicked")??;
    let delta = after.delta(&before);

    Ok((
        LoadResult {
            achieved_rate: ok as f64 / elapsed,
            failed,
            p99_latency_ms: probe.p99(),
            events_dropped: delta.events_dropped.max(0.0) as u64,
        },
        usage,
    ))
}

fn saturation_signals(
    observation: &Observation,
    baseline_p99: Option<f64>,
    cpu_threshold_percent: u16,
    available_cpus: usize,
) -> Vec<String> {
    let mut signals = Vec::new();
    if observation.active_connections < observation.target_connections {
        signals.push("load_generator_exhausted".to_string());
    }
    if observation.load.events_dropped > 0 {
        signals.push("events_dropped".to_string());
    }
    let cpu_ceiling = available_cpus.max(1) as f64 * cpu_threshold_percent as f64;
    if observation.usage.peak_cpu_percent >= cpu_ceiling {
        signals.push("cpu_pegged".to_string());
    }
    if let Some(baseline) = baseline_p99 {
        let cliff =
            (baseline * LATENCY_CLIFF_MULTIPLIER).max(baseline + LATENCY_CLIFF_MIN_INCREASE_MS);
        if observation.load.p99_latency_ms >= cliff {
            signals.push("p99_latency_cliff".to_string());
        }
    }
    if observation.load.achieved_rate < observation.intended_rate as f64 * TARGET_RATE_FRACTION {
        signals.push("target_rate_missed".to_string());
    }
    if observation.load.failed > 0 {
        signals.push("push_failures".to_string());
    }
    signals
}

fn to_report(
    name: &str,
    unit: &str,
    observation: &Observation,
    saturated: bool,
    signals: Vec<String>,
) -> SizingRunReport {
    let (recommended_requests, recommended_limits) = recommendations(observation.usage);
    SizingRunReport {
        name: name.to_string(),
        saturation_point: observation.point,
        saturation_unit: unit.to_string(),
        saturated,
        signals,
        p99_latency_ms: observation.load.p99_latency_ms,
        achieved_event_rate_per_sec: observation.load.achieved_rate,
        events_dropped: observation.load.events_dropped,
        rss: format_memory(observation.usage.peak_rss_kib),
        cpu: format_cpu(observation.usage.peak_cpu_percent),
        recommended_requests,
        recommended_limits,
    }
}

fn recommendations(usage: Usage) -> (Resources, Resources) {
    let request = Resources {
        cpu: format_cpu((usage.peak_cpu_percent * REQUEST_FRACTION).max(0.1)),
        memory: format_memory(((usage.peak_rss_kib as f64) * REQUEST_FRACTION).ceil() as u64),
    };
    let limit = Resources {
        cpu: format_cpu((usage.peak_cpu_percent * LIMIT_MULTIPLIER).max(0.1)),
        memory: format_memory(((usage.peak_rss_kib as f64) * LIMIT_MULTIPLIER).ceil() as u64),
    };
    (request, limit)
}

fn format_cpu(cpu_percent: f64) -> String {
    format!("{}m", (cpu_percent * 10.0).ceil() as u64)
}

fn format_memory(rss_kib: u64) -> String {
    format!("{}Mi", rss_kib.max(1).div_ceil(1024))
}

fn geometric_steps(start: u64, max: u64) -> Vec<u64> {
    let mut steps = vec![start];
    while *steps.last().expect("start step exists") < max {
        let current = *steps.last().expect("start step exists");
        let next = current.saturating_mul(2).min(max);
        if next == current {
            break;
        }
        steps.push(next);
    }
    steps
}

async fn wait_for_connections(target: &Target, expected: usize) -> Result<usize> {
    let http = reqwest::Client::new();
    let base = target.instance_base(target.subscriber_index());
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut active = 0usize;
    while Instant::now() < deadline {
        let snapshot = metrics::scrape(&http, base).await?;
        active = snapshot.connections_active as usize;
        let subscriptions = snapshot.subscriptions_active as usize;
        if (expected == 0 && active == 0)
            || (expected > 0 && active >= expected && subscriptions >= expected)
        {
            return Ok(active);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    Ok(active)
}

async fn sample_usage(pid: u32, mut stop: watch::Receiver<bool>) -> Result<Usage> {
    let mut usage = Usage::default();
    let mut previous = process_sample(pid).map(|sample| (Instant::now(), sample));
    loop {
        update_usage(pid, &mut usage, &mut previous);
        if *stop.borrow() {
            break;
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(200)) => {}
            changed = stop.changed() => {
                if changed.is_err() || *stop.borrow() {
                    break;
                }
            }
        }
    }
    update_usage(pid, &mut usage, &mut previous);
    ensure!(
        usage.peak_rss_kib > 0,
        "could not sample process CPU and RSS"
    );
    Ok(usage)
}

fn update_usage(pid: u32, usage: &mut Usage, previous: &mut Option<(Instant, ProcessSample)>) {
    let Some(sample) = process_sample(pid) else {
        return;
    };
    let now = Instant::now();
    usage.peak_rss_kib = usage.peak_rss_kib.max(sample.peak_rss_kib);
    if let Some((previous_at, previous_sample)) = previous {
        let elapsed = now.duration_since(*previous_at).as_secs_f64();
        if elapsed >= 0.05 {
            let cpu_percent =
                (sample.cpu_seconds - previous_sample.cpu_seconds).max(0.0) / elapsed * 100.0;
            usage.peak_cpu_percent = usage.peak_cpu_percent.max(cpu_percent);
        }
    }
    *previous = Some((now, sample));
}

#[derive(Clone, Copy)]
struct ProcessSample {
    cpu_seconds: f64,
    peak_rss_kib: u64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct RusageInfoV0 {
    ri_uuid: [u8; 16],
    ri_user_time: u64,
    ri_system_time: u64,
    ri_pkg_idle_wkups: u64,
    ri_interrupt_wkups: u64,
    ri_pageins: u64,
    ri_wired_size: u64,
    ri_resident_size: u64,
    ri_phys_footprint: u64,
    ri_proc_start_abstime: u64,
    ri_proc_exit_abstime: u64,
}

#[cfg(target_os = "macos")]
extern "C" {
    fn proc_pid_rusage(pid: c_int, flavor: c_int, buffer: *mut RusageInfoV0) -> c_int;
}

#[cfg(target_os = "macos")]
fn process_sample(pid: u32) -> Option<ProcessSample> {
    let mut usage = MaybeUninit::<RusageInfoV0>::uninit();
    // SAFETY: `usage` points to a writable RUSAGE_INFO_V0 buffer and the
    // requested child PID remains owned by `SizingServerProcess` during sampling.
    if unsafe { proc_pid_rusage(pid as c_int, 0, usage.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: proc_pid_rusage returned success and initialized `usage`.
    let usage = unsafe { usage.assume_init() };
    Some(ProcessSample {
        cpu_seconds: (usage.ri_user_time + usage.ri_system_time) as f64 / 1_000_000_000.0,
        peak_rss_kib: usage.ri_resident_size.div_ceil(1024),
    })
}

#[cfg(target_os = "linux")]
extern "C" {
    fn sysconf(name: c_int) -> c_long;
}

#[cfg(target_os = "linux")]
fn process_sample(pid: u32) -> Option<ProcessSample> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_command = stat.get(stat.rfind(')')? + 1..)?;
    let fields: Vec<&str> = after_command.split_whitespace().collect();
    let user_ticks = fields.get(11)?.parse::<u64>().ok()?;
    let system_ticks = fields.get(12)?.parse::<u64>().ok()?;
    // `_SC_CLK_TCK` is 2 on glibc/musl Linux. A positive result is the kernel clock-tick
    // frequency used by `/proc/<pid>/stat`'s user/system CPU counters.
    let ticks_per_second = unsafe { sysconf(2) };
    if ticks_per_second <= 0 {
        return None;
    }
    Some(ProcessSample {
        cpu_seconds: (user_ticks + system_ticks) as f64 / ticks_per_second as f64,
        peak_rss_kib: current_rss_kib(pid)?,
    })
}

#[cfg(target_os = "linux")]
fn current_rss_kib(pid: u32) -> Option<u64> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn process_sample(_pid: u32) -> Option<ProcessSample> {
    None
}

fn current_provenance() -> Result<(String, bool)> {
    let repository = env!("CARGO_MANIFEST_DIR");
    let output = Command::new("git")
        .args(["-C", repository, "rev-parse", "--verify", "HEAD"])
        .output()
        .context("run git rev-parse for sizing provenance")?;
    ensure!(
        output.status.success(),
        "git rev-parse failed while recording sizing provenance"
    );
    let commit = String::from_utf8(output.stdout)
        .context("git returned a non-UTF-8 commit")?
        .trim()
        .to_string();
    ensure!(
        commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "git returned an invalid commit SHA"
    );
    let status = Command::new("git")
        .args(["-C", repository, "status", "--porcelain=v1"])
        .output()
        .context("run git status for sizing provenance")?;
    ensure!(
        status.status.success(),
        "git status failed while recording sizing provenance"
    );
    Ok((commit, !status.stdout.is_empty()))
}

fn iso8601_utc(now: SystemTime) -> Result<String> {
    let seconds = now
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs();
    let days = (seconds / 86_400) as i64;
    let second_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = second_of_day / 3_600;
    let minute = (second_of_day % 3_600) / 60;
    let second = second_of_day % 60;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

// Howard Hinnant's civil-from-days algorithm, with day zero at 1970-01-01.
fn civil_from_days(days_since_epoch: i64) -> (i64, u64, u64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u64, day as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation() -> Observation {
        Observation {
            point: 400,
            intended_rate: 400,
            target_connections: 100,
            active_connections: 100,
            load: LoadResult {
                achieved_rate: 395.0,
                failed: 0,
                p99_latency_ms: 4.0,
                events_dropped: 0,
            },
            usage: Usage {
                peak_cpu_percent: 50.0,
                peak_rss_kib: 100 * 1024,
            },
        }
    }

    #[test]
    fn geometric_steps_double_and_include_the_ceiling() {
        assert_eq!(geometric_steps(100, 450), vec![100, 200, 400, 450]);
        assert_eq!(geometric_steps(7, 7), vec![7]);
    }

    #[test]
    fn saturation_combines_independent_signals() {
        let mut observed = observation();
        observed.active_connections = 99;
        observed.load.events_dropped = 2;
        observed.load.p99_latency_ms = 31.0;
        observed.load.achieved_rate = 300.0;
        observed.load.failed = 1;
        observed.usage.peak_cpu_percent = 95.0;
        assert_eq!(
            saturation_signals(&observed, Some(5.0), 90, 1),
            vec![
                "load_generator_exhausted",
                "events_dropped",
                "cpu_pegged",
                "p99_latency_cliff",
                "target_rate_missed",
                "push_failures",
            ]
        );
    }

    #[test]
    fn cpu_threshold_scales_with_available_cpus() {
        let mut observed = observation();
        observed.usage.peak_cpu_percent = 150.0;
        assert!(saturation_signals(&observed, Some(4.0), 90, 2).is_empty());
        observed.usage.peak_cpu_percent = 180.0;
        assert_eq!(
            saturation_signals(&observed, Some(4.0), 90, 2),
            vec!["cpu_pegged"]
        );
    }

    #[test]
    fn recommendations_apply_published_fractions() {
        let (requests, limits) = recommendations(Usage {
            peak_cpu_percent: 100.0,
            peak_rss_kib: 100 * 1024,
        });
        assert_eq!(
            requests,
            Resources {
                cpu: "700m".to_string(),
                memory: "70Mi".to_string(),
            }
        );
        assert_eq!(
            limits,
            Resources {
                cpu: "1500m".to_string(),
                memory: "150Mi".to_string(),
            }
        );
    }

    #[test]
    fn utc_timestamp_is_rfc3339_seconds() {
        assert_eq!(
            iso8601_utc(UNIX_EPOCH).expect("epoch formats"),
            "1970-01-01T00:00:00Z"
        );
        assert_eq!(
            iso8601_utc(UNIX_EPOCH + Duration::from_secs(951_827_696)).expect("leap day formats"),
            "2000-02-29T12:34:56Z"
        );
    }

    #[test]
    fn process_sample_reads_cpu_and_rss() {
        let sample =
            process_sample(std::process::id()).expect("current process resources are readable");
        assert!(sample.cpu_seconds >= 0.0);
        assert!(sample.peak_rss_kib > 0);
    }
}
