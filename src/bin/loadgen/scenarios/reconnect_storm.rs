//! Reconnect-storm oddity: open, hold, and drop N connections in
//! repeated waves, re-establishing the whole set each time. Confirms
//! the server cleans up every dropped connection (no leak —
//! `connections_active` settles low) and keeps admitting fresh waves.

use std::time::{Duration, Instant};

use anyhow::Result;

use crate::cli::Cli;
use crate::client::{self, Ws};
use crate::report::{Finding, OddityReport, Percentiles, Report};
use crate::scenarios::oddity_meta;
use crate::{metrics, server, token};

const STREAM: &str = "loadgen";

/// Hard ceiling on connections opened across the whole run, so the
/// storm cannot exhaust the OS ephemeral-port space on a long run —
/// each closed connection lingers in `TIME_WAIT`.
const MAX_CONNECTIONS: u64 = 5000;

pub async fn run(cli: &Cli, count: usize) -> Result<Report> {
    let http = reqwest::Client::new();
    let target = server::start_profiled(
        server::ServerProfile::default(),
        &cli.signing_key,
        &cli.push_token,
    )
    .await?;
    let base = target.instance_base(0).to_string();
    let ws_base = target.instance_ws(0).to_string();
    let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;

    let deadline = Instant::now() + Duration::from_secs(cli.duration);
    let mut waves = 0u64;
    let mut attempts = 0u64;
    let mut connected = 0u64;
    let mut resubscribe_ms = Vec::new();

    while Instant::now() < deadline && attempts < MAX_CONNECTIONS {
        // A wave: open `count` connections concurrently, holding each
        // one alive once it is up and subscribed.
        let mut tasks = Vec::with_capacity(count);
        for _ in 0..count {
            let ws_base = ws_base.clone();
            let token = token.clone();
            tasks.push(tokio::spawn(async move {
                let started = Instant::now();
                let mut ws = client::connect(&ws_base).await.ok()?;
                client::auth_and_subscribe(&mut ws, &token, "s0", STREAM, None)
                    .await
                    .ok()?;
                Some((ws, started.elapsed().as_secs_f64() * 1000.0))
            }));
        }
        let mut held: Vec<Ws> = Vec::with_capacity(count);
        for task in tasks {
            attempts += 1;
            if let Ok(Some((ws, ms))) = task.await {
                connected += 1;
                resubscribe_ms.push(ms);
                held.push(ws);
            }
        }
        // Hold the wave, then drop every connection at once and give the
        // server a moment to reap them before the next wave reconnects.
        tokio::time::sleep(Duration::from_millis(150)).await;
        drop(held);
        tokio::time::sleep(Duration::from_millis(150)).await;
        waves += 1;
    }

    // Let the server reap the final wave before sampling.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let snap = metrics::scrape(&http, &base).await?;
    let lingering = snap.connections_active as i64;
    let success_pct = if attempts == 0 {
        0.0
    } else {
        100.0 * connected as f64 / attempts as f64
    };
    let resubscribe = Percentiles::from_millis(resubscribe_ms);

    let findings = vec![
        Finding::new(
            "connection success rate",
            "≥ 99%",
            format!("{success_pct:.1}% ({connected}/{attempts})"),
            success_pct >= 99.0,
        ),
        Finding::new(
            "reconnect latency",
            "p99 < 1000ms",
            format!("p50={:.0}ms p99={:.0}ms", resubscribe.p50, resubscribe.p99),
            resubscribe.p99 < 1000.0,
        ),
        Finding::new(
            "no connection leak",
            format!("< {count} active after the storm"),
            lingering.to_string(),
            lingering < count as i64,
        ),
        Finding::new("waves completed", "> 0", waves.to_string(), waves > 0),
    ];
    Ok(Report::Oddity(OddityReport::new(
        oddity_meta(cli, "reconnect-storm"),
        format!("{count} connections opened, held, dropped, and re-established in waves"),
        findings,
    )))
}
