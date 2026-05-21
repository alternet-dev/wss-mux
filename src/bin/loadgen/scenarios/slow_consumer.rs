//! Slow-consumer oddity: a subscriber stops reading on a small-queue
//! stream while events burst. Confirms it overflows on its own — a
//! keep-open `overflow` error and a non-zero `events_dropped{overflow}`
//! — while a healthy subscriber on the same stream keeps receiving.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;

use crate::cli::Cli;
use crate::client::{self, Recv};
use crate::report::{Finding, OddityReport, Report};
use crate::scenarios::{oddity_meta, post_event, wait_for_subscriptions};
use crate::{metrics, server, token};

const SLOW_STREAM: &str = "loadgen_slow";

pub async fn run(cli: &Cli) -> Result<Report> {
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

    // A healthy subscriber that drains continuously.
    let mut healthy = client::connect(&ws_base).await?;
    client::auth_and_subscribe(&mut healthy, &token, "healthy", SLOW_STREAM, None).await?;
    // A slow subscriber that subscribes, then never reads.
    let mut slow = client::connect(&ws_base).await?;
    client::auth_and_subscribe(&mut slow, &token, "slow", SLOW_STREAM, None).await?;
    wait_for_subscriptions(&http, &[base.as_str()], 2).await?;

    let deadline = Instant::now() + Duration::from_secs(cli.duration);
    let healthy_count = Arc::new(AtomicU64::new(0));
    let drain = {
        let healthy_count = Arc::clone(&healthy_count);
        tokio::spawn(async move {
            while Instant::now() < deadline {
                match client::next(&mut healthy, Duration::from_millis(250)).await {
                    Ok(Recv::Event(_)) => {
                        healthy_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Ok(Recv::Closed) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        })
    };

    // Burst events at the small-queue stream — far more than depth 8.
    let body = serde_json::to_vec(&json!({
        "stream": SLOW_STREAM,
        "payload": {"loadgen": true},
    }))?;
    while Instant::now() < deadline {
        let _ = post_event(&http, &base, &cli.push_token, &body).await;
    }
    drain.abort();

    // The slow subscriber's socket holds a few buffered events, then a
    // keep-open `overflow` error. Drain it and look for that error.
    let mut slow_overflow = false;
    let mut slow_closed = false;
    loop {
        match client::next(&mut slow, Duration::from_millis(500)).await {
            Ok(Recv::Error { code }) if code == "overflow" => {
                slow_overflow = true;
                break;
            }
            Ok(Recv::Idle) => break,
            Ok(Recv::Closed) | Err(_) => {
                slow_closed = true;
                break;
            }
            Ok(_) => {}
        }
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    let snap = metrics::scrape(&http, &base).await?;
    let delivered = healthy_count.load(Ordering::Relaxed);
    let overflow_drops = snap.overflow_drops as u64;

    let findings = vec![
        Finding::new(
            "slow subscriber overflow error",
            "received (code=overflow)",
            if slow_overflow {
                "received"
            } else {
                "not received"
            },
            slow_overflow,
        ),
        Finding::new(
            "slow connection stays open",
            "open (overflow is keep-open)",
            if slow_closed { "closed" } else { "open" },
            !slow_closed,
        ),
        Finding::new(
            "events_dropped{overflow}",
            "> 0",
            overflow_drops.to_string(),
            overflow_drops > 0,
        ),
        Finding::new(
            "healthy subscriber keeps delivering",
            "> 0 events",
            delivered.to_string(),
            delivered > 0,
        ),
    ];
    Ok(Report::Oddity(OddityReport::new(
        oddity_meta(cli, "slow-consumer"),
        "a subscriber that stops reading on a queue_depth=8 stream while events burst",
        findings,
    )))
}
