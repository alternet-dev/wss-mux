//! Latency: a paced producer stamps each event; subscribers measure the
//! push→deliver time. With a fleet the path includes the cross-instance
//! relay hop; `--topology` decides subscriber and key placement.

use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;
use wss_mux::envelope::ServerFrame;

use crate::cli::Cli;
use crate::client::{self, Recv};
use crate::metrics;
use crate::report::{LatencyReport, Percentiles, RelayStats, Report, RunMeta};
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
    let producer_base = target.producer_base().to_string();
    let instance_bases = target.instance_bases();
    let token = token::mint_token(&cli.signing_key, &["role:loadgen"])?;
    let subscribers = subscribers.max(1);

    // One process-wide clock origin: producer and subscribers run in
    // this one process (even in external mode), so an `Instant` offset
    // stamped into the payload is directly comparable on receipt.
    let origin = Instant::now();

    // Place each subscriber per the topology, then wait for the server
    // to register them before the producer starts pushing.
    let plan =
        cli.topology
            .subscriber_plan(subscribers, target.fleet_size(), target.subscriber_index());
    let mut subs = Vec::with_capacity(subscribers);
    for (i, placement) in plan.iter().enumerate() {
        let mut ws = client::connect(target.instance_ws(placement.instance)).await?;
        client::auth_and_subscribe(
            &mut ws,
            &token,
            &format!("s{i}"),
            &cli.stream,
            placement.key.as_deref(),
        )
        .await?;
        subs.push(ws);
    }
    wait_for_subscriptions(&http, &instance_bases, subscribers).await?;

    let before = metrics::scrape_fleet(&http, &instance_bases).await?;
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

    // Paced producer: one stamped event per tick, cycling the keys.
    let producer = {
        let http = http.clone();
        let base = producer_base.clone();
        let push_token = cli.push_token.clone();
        let stream = cli.stream.clone();
        let keys = cli.topology.producer_keys(subscribers);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms.max(1)));
            let mut cursor = 0usize;
            while Instant::now() < deadline {
                ticker.tick().await;
                let stamp = origin.elapsed().as_nanos() as u64;
                let key = &keys[cursor % keys.len()];
                cursor += 1;
                let body = match key {
                    Some(k) => json!({"stream": stream, "key": k, "payload": {"t_nanos": stamp}}),
                    None => json!({"stream": stream, "payload": {"t_nanos": stamp}}),
                };
                let Ok(bytes) = serde_json::to_vec(&body) else {
                    continue;
                };
                let _ = post_event(&http, &base, &push_token, &bytes).await;
            }
        })
    };

    let _ = producer.await;
    let mut latency_ms = Vec::new();
    for task in sub_tasks {
        latency_ms.extend(task.await.expect("subscriber task panicked"));
    }

    // Let in-flight cross-instance relays land before the final scrape.
    if target.peers() > 0 {
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    let delta = metrics::scrape_fleet(&http, &instance_bases)
        .await?
        .delta(&before);
    let relay = if target.peers() > 0 {
        Some(RelayStats::new(
            delta.relay_events_relayed as u64,
            delta.relay_sent as u64,
            delta.relay_events_unwanted as u64,
        ))
    } else {
        None
    };

    Ok(Report::Latency(LatencyReport {
        meta: RunMeta {
            scenario: "latency".to_string(),
            mode: target.mode().to_string(),
            topology: cli.topology.as_str().to_string(),
            peers: target.peers(),
            duration_secs: cli.duration,
        },
        subscribers,
        interval_ms,
        samples: latency_ms.len(),
        delivery_latency_ms: Percentiles::from_millis(latency_ms),
        relay,
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
