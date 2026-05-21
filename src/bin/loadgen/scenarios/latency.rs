//! Latency: a paced producer stamps each event; subscribers measure the
//! push→deliver time.

use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;
use wss_mux::envelope::ServerFrame;

use crate::cli::Cli;
use crate::client::{self, Recv};
use crate::report::{LatencyReport, Percentiles, Report, RunMeta};
use crate::scenarios::{post_event, wait_for_subscriptions};
use crate::server::Target;
use crate::token;

pub async fn run(
    cli: &Cli,
    target: &Target,
    subscribers: usize,
    interval_ms: u64,
) -> Result<Report> {
    let http = reqwest::Client::new();
    let base = target.base_url();
    let ws_base = target.ws_url();
    let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;
    let subscribers = subscribers.max(1);

    // One process-wide clock origin: producer and subscribers run in
    // this one process (even in external mode), so an `Instant` offset
    // stamped into the payload is directly comparable on receipt.
    let origin = Instant::now();

    // Connect the subscribers (send auth + subscribe), then wait for the
    // server to register them before the producer starts pushing.
    let mut subs = Vec::with_capacity(subscribers);
    for i in 0..subscribers {
        let mut ws = client::connect(&ws_base).await?;
        client::auth_and_subscribe(&mut ws, &token, &format!("s{i}"), &cli.stream).await?;
        subs.push(ws);
    }
    wait_for_subscriptions(&http, &base, subscribers).await?;

    let deadline = Instant::now() + Duration::from_secs(cli.duration);

    // Each subscriber measures its own deliveries.
    let mut sub_tasks = Vec::with_capacity(subscribers);
    for mut ws in subs {
        sub_tasks.push(tokio::spawn(async move {
            let mut samples = Vec::new();
            while Instant::now() < deadline {
                match client::next(&mut ws, Duration::from_millis(250)).await {
                    Ok(Recv::Event(frame)) => {
                        if let Some(ms) = delivery_latency_ms(&frame, origin) {
                            samples.push(ms);
                        }
                    }
                    Ok(Recv::Closed) | Err(_) => break,
                    Ok(_) => {}
                }
            }
            samples
        }));
    }

    // Paced producer: one stamped event per tick.
    let producer = {
        let http = http.clone();
        let base = base.clone();
        let push_token = cli.push_token.clone();
        let stream = cli.stream.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms.max(1)));
            while Instant::now() < deadline {
                ticker.tick().await;
                let stamp = origin.elapsed().as_nanos() as u64;
                let Ok(body) = serde_json::to_vec(&json!({
                    "stream": stream,
                    "payload": {"t_nanos": stamp},
                })) else {
                    continue;
                };
                let _ = post_event(&http, &base, &push_token, &body).await;
            }
        })
    };

    let _ = producer.await;
    let mut latency_ms = Vec::new();
    for task in sub_tasks {
        latency_ms.extend(task.await.expect("subscriber task panicked"));
    }

    Ok(Report::Latency(LatencyReport {
        meta: RunMeta {
            scenario: "latency".to_string(),
            mode: target.mode().to_string(),
            duration_secs: cli.duration,
        },
        subscribers,
        interval_ms,
        samples: latency_ms.len(),
        delivery_latency_ms: Percentiles::from_millis(latency_ms),
    }))
}

/// Extract the producer stamp from an event payload and turn it into a
/// push→deliver latency in milliseconds.
fn delivery_latency_ms(frame: &ServerFrame, origin: Instant) -> Option<f64> {
    let ServerFrame::Event { payload, .. } = frame else {
        return None;
    };
    let stamp = payload.get("t_nanos")?.as_u64()?;
    let now = origin.elapsed().as_nanos() as u64;
    Some(now.saturating_sub(stamp) as f64 / 1_000_000.0)
}
