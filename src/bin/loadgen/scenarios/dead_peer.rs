//! Dead-peer oddity: producer pushes relayed to a guaranteed-closed
//! peer port. Confirms relay failures are metered
//! (`relay_failed{connect}`) and the producer is never back-pressured —
//! a dead peer must not slow ingest or affect local delivery.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;

use crate::cli::Cli;
use crate::client::{self, Recv};
use crate::report::{Finding, OddityReport, Percentiles, Report};
use crate::scenarios::{oddity_meta, post_event, wait_for_subscriptions};
use crate::{metrics, server, token};

const STREAM: &str = "loadgen";

pub async fn run(cli: &Cli) -> Result<Report> {
    let http = reqwest::Client::new();
    let profile = server::ServerProfile {
        dead_peer: true,
        ..server::ServerProfile::default()
    };
    let target = server::start_profiled(profile, &cli.signing_key, &cli.push_token).await?;
    let base = target.instance_base(0).to_string();
    let ws_base = target.instance_ws(0).to_string();
    let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;

    // A local subscriber, draining — local delivery must be unaffected.
    let mut sub = client::connect(&ws_base).await?;
    client::auth_and_subscribe(&mut sub, &token, "s0", STREAM, None).await?;
    wait_for_subscriptions(&http, &[base.as_str()], 1).await?;

    let deadline = Instant::now() + Duration::from_secs(cli.duration);
    let delivered = Arc::new(AtomicU64::new(0));
    let drain = {
        let delivered = Arc::clone(&delivered);
        tokio::spawn(async move {
            while Instant::now() < deadline {
                match client::next(&mut sub, Duration::from_millis(250)).await {
                    Ok(Recv::Event(_)) => {
                        delivered.fetch_add(1, Ordering::Relaxed);
                    }
                    Ok(Recv::Closed) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        })
    };

    // Burst: every push relays to the dead peer and fails fast.
    let body = serde_json::to_vec(&json!({
        "stream": STREAM,
        "payload": {"loadgen": true},
    }))?;
    let mut pushes_ok = 0u64;
    let mut pushes_failed = 0u64;
    let mut latency_ms = Vec::new();
    while Instant::now() < deadline {
        let started = Instant::now();
        match post_event(&http, &base, &cli.push_token, &body).await {
            Ok(status) if status.is_success() => pushes_ok += 1,
            _ => pushes_failed += 1,
        }
        latency_ms.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    drain.abort();

    tokio::time::sleep(Duration::from_millis(300)).await;
    let snap = metrics::scrape(&http, &base).await?;
    let relay_failed = snap.relay_failed_connect as u64;
    let push_p99 = Percentiles::from_millis(latency_ms).p99;
    let delivered = delivered.load(Ordering::Relaxed);

    let findings = vec![
        Finding::new(
            "relay_failed{connect}",
            "> 0",
            relay_failed.to_string(),
            relay_failed > 0,
        ),
        Finding::new(
            "producer never back-pressured",
            "all pushes 204",
            format!("{pushes_ok} ok / {pushes_failed} failed"),
            pushes_failed == 0,
        ),
        Finding::new(
            "producer POST p99",
            "< 50ms",
            format!("{push_p99:.2}ms"),
            push_p99 < 50.0,
        ),
        Finding::new(
            "local delivery intact",
            "> 0 events",
            delivered.to_string(),
            delivered > 0,
        ),
    ];
    Ok(Report::Oddity(OddityReport::new(
        oddity_meta(cli, "dead-peer"),
        "producer pushes relayed to a guaranteed-closed peer port",
        findings,
    )))
}
