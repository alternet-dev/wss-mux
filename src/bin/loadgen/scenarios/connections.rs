//! Connections: open and hold N connections, report the sustained
//! active counts the server itself reports.

use std::time::{Duration, Instant};

use anyhow::Result;

use crate::cli::Cli;
use crate::client::{self, Recv};
use crate::metrics;
use crate::report::{ConnectionsReport, Report, RunMeta};
use crate::scenarios::wait_for_subscriptions;
use crate::server::Target;
use crate::token;

pub async fn run(cli: &Cli, target: &Target, count: usize) -> Result<Report> {
    let http = reqwest::Client::new();
    let base = target.subscriber_base().to_string();
    let ws_base = target.subscriber_ws().to_string();
    let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;

    let deadline = Instant::now() + Duration::from_secs(cli.duration);

    // One task per connection: connect, auth + subscribe, then drain
    // until the window closes (draining keeps tungstenite answering
    // pings). Each task reports whether its connection came up.
    let mut holders = Vec::with_capacity(count);
    for _ in 0..count {
        let ws_base = ws_base.clone();
        let token = token.clone();
        let stream = cli.stream.clone();
        holders.push(tokio::spawn(async move {
            let mut ws = match client::connect(&ws_base).await {
                Ok(ws) => ws,
                Err(_) => return false,
            };
            if client::auth_and_subscribe(&mut ws, &token, "s0", &stream)
                .await
                .is_err()
            {
                return false;
            }
            while Instant::now() < deadline {
                match client::next(&mut ws, Duration::from_millis(500)).await {
                    Ok(Recv::Closed) | Err(_) => return true,
                    Ok(_) => {}
                }
            }
            true
        }));
    }

    // Let the connections reach the server, then sample the sustained
    // counts around the middle of the hold window.
    let _ = wait_for_subscriptions(&http, &base, count).await;
    tokio::time::sleep(hold_midpoint(deadline)).await;
    let snap = metrics::scrape(&http, &base).await?;
    let rss_kib = if target.mode() == "in-process" {
        rss_kib()
    } else {
        None
    };

    // Drain the rest of the window, then collect how many came up.
    let mut connected = 0usize;
    for holder in holders {
        if holder.await.unwrap_or(false) {
            connected += 1;
        }
    }

    Ok(Report::Connections(ConnectionsReport {
        meta: RunMeta {
            scenario: "connections".to_string(),
            mode: target.mode().to_string(),
            peers: target.peers(),
            duration_secs: cli.duration,
        },
        requested: count,
        connected,
        connections_active: snap.connections_active as i64,
        subscriptions_active: snap.subscriptions_active as i64,
        rss_kib,
    }))
}

/// How long to wait before sampling the sustained counts: half the time
/// left until `deadline`, clamped so it is never zero.
fn hold_midpoint(deadline: Instant) -> Duration {
    let remaining = deadline.saturating_duration_since(Instant::now());
    (remaining / 2).max(Duration::from_millis(100))
}

/// Resident set size of this process, in KiB, via `ps`. Best-effort —
/// any failure yields `None`.
fn rss_kib() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}
