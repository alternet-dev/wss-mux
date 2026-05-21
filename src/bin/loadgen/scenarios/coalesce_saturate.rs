//! Coalesce-saturation oddity: burst past a small relay-coalescing
//! queue. Confirms over-quota batches are dropped and metered
//! (`relay_queue_dropped`), the flush worker keeps cycling
//! (`relay_flushes`), and the producer sees no back-pressure.

use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;

use crate::cli::Cli;
use crate::report::{Finding, OddityReport, Report};
use crate::scenarios::{oddity_meta, post_event};
use crate::{metrics, server};

const STREAM: &str = "loadgen";

pub async fn run(cli: &Cli) -> Result<Report> {
    let http = reqwest::Client::new();
    let profile = server::ServerProfile {
        coalesce_ms: 50,
        relay_queue_depth: 16,
        dead_peer: true,
        ..server::ServerProfile::default()
    };
    let target = server::start_profiled(profile, &cli.signing_key, &cli.push_token).await?;
    let base = target.instance_base(0).to_string();

    // Burst hard from a pool of producers — far faster than a 50ms
    // window can drain a depth-16 coalescing queue.
    let deadline = Instant::now() + Duration::from_secs(cli.duration);
    let body = serde_json::to_vec(&json!({
        "stream": STREAM,
        "payload": {"loadgen": true},
    }))?;

    let mut producers = Vec::with_capacity(8);
    for _ in 0..8 {
        let http = http.clone();
        let base = base.clone();
        let push_token = cli.push_token.clone();
        let body = body.clone();
        producers.push(tokio::spawn(async move {
            let mut ok = 0u64;
            let mut failed = 0u64;
            while Instant::now() < deadline {
                match post_event(&http, &base, &push_token, &body).await {
                    Ok(status) if status.is_success() => ok += 1,
                    _ => failed += 1,
                }
            }
            (ok, failed)
        }));
    }
    let mut pushes_ok = 0u64;
    let mut pushes_failed = 0u64;
    for producer in producers {
        let (ok, failed) = producer.await.expect("producer task panicked");
        pushes_ok += ok;
        pushes_failed += failed;
    }

    // Let the flush worker run a few more cycles, then read the totals.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let snap = metrics::scrape(&http, &base).await?;
    let dropped = snap.relay_queue_dropped as u64;
    let flushes = snap.relay_flushes as u64;

    let findings = vec![
        Finding::new(
            "relay_queue_dropped",
            "> 0",
            dropped.to_string(),
            dropped > 0,
        ),
        Finding::new(
            "relay_flushes advancing",
            "> 0",
            flushes.to_string(),
            flushes > 0,
        ),
        Finding::new(
            "producer never back-pressured",
            "all pushes 204",
            format!("{pushes_ok} ok / {pushes_failed} failed"),
            pushes_failed == 0,
        ),
    ];
    Ok(Report::Oddity(OddityReport::new(
        oddity_meta(cli, "coalesce-saturate"),
        "a producer burst past a depth-16 relay-coalescing queue (50ms window)",
        findings,
    )))
}
