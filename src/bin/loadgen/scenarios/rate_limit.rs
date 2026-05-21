//! Rate-limit oddity: flood inbound client frames past a low rate
//! limit. Confirms throttled frames draw keep-open `rate_limited`
//! errors, the counter climbs, the connection survives, and after a
//! refill window admission resumes.

use std::time::Duration;

use anyhow::Result;
use wss_mux::envelope::ClientFrame;

use crate::cli::Cli;
use crate::client::{self, Recv};
use crate::report::{Finding, OddityReport, Report};
use crate::scenarios::oddity_meta;
use crate::{metrics, server, token};

const STREAM: &str = "loadgen";

pub async fn run(cli: &Cli) -> Result<Report> {
    let http = reqwest::Client::new();
    // 5 frames/sec, burst 10 — a flood of 200 frames is overwhelmingly
    // throttled, but the budget is wide enough not to be fiddly.
    let profile = server::ServerProfile {
        rate_limit: (5, 10),
        ..server::ServerProfile::default()
    };
    let target = server::start_profiled(profile, &cli.signing_key, &cli.push_token).await?;
    let base = target.instance_base(0).to_string();
    let ws_base = target.instance_ws(0).to_string();
    let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;

    let mut ws = client::connect(&ws_base).await?;
    client::send_frame(&mut ws, &ClientFrame::Auth { token }).await?;

    // Flood idempotent unsubscribe frames; once the bucket drains, every
    // further frame draws a keep-open `rate_limited` error.
    for i in 0..200 {
        client::send_frame(
            &mut ws,
            &ClientFrame::Unsubscribe {
                id: format!("flood-{i}"),
            },
        )
        .await?;
    }

    // Drain the socket, counting `rate_limited` errors.
    let mut rate_limited = 0u64;
    let mut closed = false;
    loop {
        match client::next(&mut ws, Duration::from_millis(500)).await {
            Ok(Recv::Error { code }) if code == "rate_limited" => rate_limited += 1,
            Ok(Recv::Idle) => break,
            Ok(Recv::Closed) | Err(_) => {
                closed = true;
                break;
            }
            Ok(_) => {}
        }
    }

    // After a refill window a fresh subscribe should be admitted.
    tokio::time::sleep(Duration::from_secs(3)).await;
    client::send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "recovered".to_string(),
            stream: STREAM.to_string(),
            key: None,
        },
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let snap = metrics::scrape(&http, &base).await?;
    let throttled = snap.frames_rate_limited as u64;
    let readmitted = snap.subscriptions_active as i64 >= 1;

    let findings = vec![
        Finding::new(
            "rate_limited error frames",
            "> 0",
            rate_limited.to_string(),
            rate_limited > 0,
        ),
        Finding::new(
            "frames_rate_limited metered",
            "> 0",
            throttled.to_string(),
            throttled > 0,
        ),
        Finding::new(
            "connection survives throttling",
            "stays open",
            if closed { "closed" } else { "open" },
            !closed,
        ),
        Finding::new(
            "admission resumes after refill",
            "subscribe admitted",
            if readmitted {
                "admitted"
            } else {
                "still throttled"
            },
            readmitted,
        ),
    ];
    Ok(Report::Oddity(OddityReport::new(
        oddity_meta(cli, "rate-limit"),
        "200 inbound frames flooded past a 5/sec (burst 10) rate limit",
        findings,
    )))
}
