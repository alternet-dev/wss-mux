//! `wss-mux-loadgen` — a load and stress harness for wss-mux.
//!
//! Drives throughput, latency, and connection-count measurements
//! against either an in-process or an external wss-mux instance, and
//! prints a human or `--json` report.

mod cli;
mod client;
mod metrics;
mod report;
mod scenarios;
mod server;
mod token;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Scenario};

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
    let target = server::start(
        cli.target.clone(),
        cli.peers,
        &cli.signing_key,
        &cli.push_token,
    )
    .await?;

    let report = match &cli.scenario {
        Scenario::Throughput {
            subscribers,
            producers,
        } => scenarios::throughput::run(&cli, &target, *subscribers, *producers).await?,
        Scenario::Latency {
            subscribers,
            interval_ms,
        } => scenarios::latency::run(&cli, &target, *subscribers, *interval_ms).await?,
        Scenario::Connections { count } => {
            scenarios::connections::run(&cli, &target, *count).await?
        }
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{report}");
    }
    Ok(())
}
