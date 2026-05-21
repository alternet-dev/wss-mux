//! `wss-mux-loadgen` — a load and stress harness for wss-mux.
//!
//! Drives throughput, latency, connection-count, and traffic-oddity
//! measurements against an in-process or external wss-mux instance, and
//! prints a human or `--json` report.

mod cli;
mod client;
mod metrics;
mod report;
mod scenarios;
mod server;
mod token;
mod topology;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Scenario};
use crate::server::Target;

#[tokio::main]
async fn main() -> Result<()> {
    // Harness logs go to stderr so `--json` keeps stdout a clean,
    // parseable single object. Quiet by default; `RUST_LOG` opens it up.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let report = match &cli.scenario {
        Scenario::Throughput {
            subscribers,
            producers,
        } => {
            let target = fleet(&cli).await?;
            scenarios::throughput::run(&cli, &target, *subscribers, *producers).await?
        }
        Scenario::Latency {
            subscribers,
            interval_ms,
        } => {
            let target = fleet(&cli).await?;
            scenarios::latency::run(&cli, &target, *subscribers, *interval_ms).await?
        }
        Scenario::Connections { count } => {
            let target = fleet(&cli).await?;
            scenarios::connections::run(&cli, &target, *count).await?
        }
        Scenario::SlowConsumer => scenarios::slow_consumer::run(&cli).await?,
        Scenario::DeadPeer => scenarios::dead_peer::run(&cli).await?,
        Scenario::CoalesceSaturate => scenarios::coalesce_saturate::run(&cli).await?,
        Scenario::ReconnectStorm { count } => scenarios::reconnect_storm::run(&cli, *count).await?,
        Scenario::PayloadCap => scenarios::payload_cap::run(&cli).await?,
        Scenario::RateLimit => scenarios::rate_limit::run(&cli).await?,
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{report}");
    }
    Ok(())
}

/// Bring up the in-process (or external) fleet the throughput, latency,
/// and connections scenarios drive. The traffic-oddity scenarios
/// instead provision their own profiled instance.
async fn fleet(cli: &Cli) -> Result<Target> {
    server::start(
        cli.target.clone(),
        cli.peers,
        &cli.signing_key,
        &cli.push_token,
    )
    .await
}
