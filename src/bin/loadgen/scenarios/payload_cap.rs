//! Payload-cap oddity: POST payloads over a stream's `max_payload_bytes`
//! cap. Confirms each oversized push is rejected with 413 and metered
//! (`events_rejected{payload_too_large}`), while an in-cap push is 204.

use anyhow::Result;
use serde_json::json;

use crate::cli::Cli;
use crate::report::{Finding, OddityReport, Report};
use crate::scenarios::{oddity_meta, post_event};
use crate::{metrics, server};

const CAPPED_STREAM: &str = "loadgen_capped";

pub async fn run(cli: &Cli) -> Result<Report> {
    let http = reqwest::Client::new();
    let target = server::start_profiled(
        server::ServerProfile::default(),
        &cli.signing_key,
        &cli.push_token,
    )
    .await?;
    let base = target.instance_base(0).to_string();

    // The capped stream caps payloads at 256 JSON bytes.
    let oversized = serde_json::to_vec(&json!({
        "stream": CAPPED_STREAM,
        "payload": {"blob": "x".repeat(512)},
    }))?;
    let in_cap = serde_json::to_vec(&json!({
        "stream": CAPPED_STREAM,
        "payload": {"ok": true},
    }))?;

    let oversized_status = post_event(&http, &base, &cli.push_token, &oversized).await?;
    let in_cap_status = post_event(&http, &base, &cli.push_token, &in_cap).await?;
    // A handful more oversized pushes so the counter is clearly nonzero.
    for _ in 0..20 {
        let _ = post_event(&http, &base, &cli.push_token, &oversized).await;
    }

    let snap = metrics::scrape(&http, &base).await?;
    let rejected = snap.events_rejected_payload as u64;
    let over_413 = oversized_status == reqwest::StatusCode::PAYLOAD_TOO_LARGE;
    let in_cap_204 = in_cap_status == reqwest::StatusCode::NO_CONTENT;

    let findings = vec![
        Finding::new(
            "oversized push rejected",
            "413 Payload Too Large",
            oversized_status.as_u16().to_string(),
            over_413,
        ),
        Finding::new(
            "in-cap push accepted",
            "204 No Content",
            in_cap_status.as_u16().to_string(),
            in_cap_204,
        ),
        Finding::new(
            "events_rejected{payload_too_large}",
            "> 0",
            rejected.to_string(),
            rejected > 0,
        ),
    ];
    Ok(Report::Oddity(OddityReport::new(
        oddity_meta(cli, "payload-cap"),
        "payloads pushed over a stream's 256-byte max_payload_bytes cap",
        findings,
    )))
}
