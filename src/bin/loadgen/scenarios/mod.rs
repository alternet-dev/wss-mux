//! Load and stress scenarios, plus the helpers they share.

pub mod coalesce_saturate;
pub mod connections;
pub mod dead_peer;
pub mod latency;
pub mod payload_cap;
pub mod rate_limit;
pub mod reconnect_storm;
pub mod slow_consumer;
pub mod throughput;

use std::time::{Duration, Instant};

use anyhow::{bail, Result};

use crate::cli::Cli;
use crate::metrics;
use crate::report::RunMeta;

/// `RunMeta` for an in-process traffic-oddity scenario — topology and
/// peer count do not apply, so they carry their inert defaults.
pub fn oddity_meta(cli: &Cli, scenario: &str) -> RunMeta {
    RunMeta {
        scenario: scenario.to_string(),
        mode: "in-process".to_string(),
        topology: cli.topology.as_str().to_string(),
        peers: 0,
        duration_secs: cli.duration,
    }
}

/// Poll the fleet's `/metrics` until `wss_mux_subscriptions_active`,
/// summed across `bases`, reaches `expected`, or fail after ~10s. The
/// black-box readiness gate before a producer starts pushing — there is
/// no per-frame ack.
pub async fn wait_for_subscriptions(
    http: &reqwest::Client,
    bases: &[&str],
    expected: usize,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last = 0i64;
    while Instant::now() < deadline {
        last = metrics::scrape_fleet(http, bases)
            .await?
            .subscriptions_active as i64;
        if last >= expected as i64 {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    bail!(
        "only {last} of {expected} subscriptions registered within 10s — check the \
         signing key and that the stream exists in the server's manifest"
    );
}

/// POST one event, sending a pre-serialized JSON body via `.body()` —
/// deliberately not `reqwest`'s `.json()`, so the harness builds against
/// the production `reqwest` feature set (which has no `json`).
pub async fn post_event(
    http: &reqwest::Client,
    base_url: &str,
    push_token: &str,
    body: &[u8],
) -> Result<reqwest::StatusCode> {
    let response = http
        .post(format!("{base_url}/v1/events"))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {push_token}"),
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_vec())
        .send()
        .await?;
    Ok(response.status())
}
