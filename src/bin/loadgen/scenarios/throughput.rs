//! Throughput: hold N subscribers, push as fast as a pool of producers
//! can for the measurement window, report the ingest ceiling. With a
//! fleet, producers push to the ingress instance and subscribers are
//! held on another, so the measured path crosses a peer-relay hop.

use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;

use crate::cli::Cli;
use crate::client;
use crate::metrics;
use crate::report::{Percentiles, RelayStats, Report, RunMeta, ThroughputReport};
use crate::scenarios::{post_event, wait_for_subscriptions};
use crate::server::Target;
use crate::token;

/// Cap on push-latency samples kept per producer task — a long run at a
/// high push rate would otherwise grow the sample vector without bound.
const MAX_LATENCY_SAMPLES: usize = 200_000;

struct ProducerResult {
    ok: u64,
    failed: u64,
    latency_ms: Vec<f64>,
}

pub async fn run(
    cli: &Cli,
    target: &Target,
    subscribers: usize,
    producers: usize,
) -> Result<Report> {
    let http = reqwest::Client::new();
    let producer_base = target.producer_base().to_string();
    let subscriber_base = target.subscriber_base().to_string();
    let subscriber_ws = target.subscriber_ws().to_string();
    let instance_bases = target.instance_bases();
    let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;
    let producers = producers.max(1);

    let deadline = Instant::now() + Duration::from_secs(cli.duration);

    // Hold `subscribers` draining connections on the subscriber instance
    // so per-sub channels never fill — a full channel would be overflow,
    // not throughput.
    let mut drains = Vec::with_capacity(subscribers);
    for i in 0..subscribers {
        let mut ws = client::connect(&subscriber_ws).await?;
        client::auth_and_subscribe(&mut ws, &token, &format!("s{i}"), &cli.stream).await?;
        drains.push(tokio::spawn(async move {
            while Instant::now() < deadline {
                match client::next(&mut ws, Duration::from_secs(1)).await {
                    Ok(client::Recv::Closed) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        }));
    }

    wait_for_subscriptions(&http, &subscriber_base, subscribers).await?;

    // A constant pre-serialized body — the harness must not bottleneck
    // on JSON encoding while it is measuring the server.
    let body = serde_json::to_vec(&json!({
        "stream": cli.stream,
        "payload": {"loadgen": true},
    }))?;

    let before = metrics::scrape_fleet(&http, &instance_bases).await?;

    let mut producer_tasks = Vec::with_capacity(producers);
    for _ in 0..producers {
        let http = http.clone();
        let base = producer_base.clone();
        let push_token = cli.push_token.clone();
        let body = body.clone();
        producer_tasks.push(tokio::spawn(async move {
            let mut result = ProducerResult {
                ok: 0,
                failed: 0,
                latency_ms: Vec::new(),
            };
            while Instant::now() < deadline {
                let started = Instant::now();
                match post_event(&http, &base, &push_token, &body).await {
                    Ok(status) if status.is_success() => result.ok += 1,
                    _ => result.failed += 1,
                }
                if result.latency_ms.len() < MAX_LATENCY_SAMPLES {
                    result
                        .latency_ms
                        .push(started.elapsed().as_secs_f64() * 1000.0);
                }
            }
            result
        }));
    }

    let mut pushes_ok = 0u64;
    let mut pushes_failed = 0u64;
    let mut latency_ms = Vec::new();
    for task in producer_tasks {
        let result = task.await.expect("producer task panicked");
        pushes_ok += result.ok;
        pushes_failed += result.failed;
        latency_ms.extend(result.latency_ms);
    }

    // Let in-flight cross-instance relays land before the final scrape.
    if target.peers() > 0 {
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    let after = metrics::scrape_fleet(&http, &instance_bases).await?;
    for drain in drains {
        drain.abort();
    }

    let delta = after.delta(&before);
    let secs = cli.duration.max(1) as f64;
    let relay = if target.peers() > 0 {
        Some(RelayStats::new(
            delta.relay_events_relayed as u64,
            delta.relay_sent as u64,
            delta.relay_events_unwanted as u64,
        ))
    } else {
        None
    };
    Ok(Report::Throughput(ThroughputReport {
        meta: RunMeta {
            scenario: "throughput".to_string(),
            mode: target.mode().to_string(),
            peers: target.peers(),
            duration_secs: cli.duration,
        },
        subscribers,
        producers,
        pushes_ok,
        pushes_failed,
        push_rate_per_sec: pushes_ok as f64 / secs,
        events_delivered: delta.events_dispatched as u64,
        delivery_rate_per_sec: delta.events_dispatched / secs,
        overflow_drops: delta.overflow_drops as u64,
        push_latency_ms: Percentiles::from_millis(latency_ms),
        relay,
    }))
}
