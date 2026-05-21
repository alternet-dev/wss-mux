//! Command-line surface for the load/stress harness.

use clap::{Parser, Subcommand};

/// Load and stress harness for wss-mux.
///
/// With no `--target`, a wss-mux instance is spawned in-process on an
/// ephemeral `127.0.0.1` port — the run is self-contained. With
/// `--target`, an already-running instance is driven instead (the true
/// ceiling: load generator and server in separate processes). Every
/// result stamps which mode produced it.
#[derive(Debug, Parser)]
#[command(name = "wss-mux-loadgen", version, about)]
pub struct Cli {
    /// Drive an already-running instance at this base URL
    /// (e.g. `http://127.0.0.1:8080`). Omitted ⇒ spawn in-process.
    #[arg(long, global = true)]
    pub target: Option<String>,

    /// HS256 handshake key used to mint subscriber tokens. In external
    /// mode it must match the server's `WSS_MUX_HANDSHAKE_SIGNING_KEY`.
    #[arg(long, global = true, default_value = "loadgen-signing-key")]
    pub signing_key: String,

    /// Producer push token. In external mode it must match the server's
    /// `WSS_MUX_PUSH_AUTH_TOKEN`.
    #[arg(long, global = true, default_value = "loadgen-push-token")]
    pub push_token: String,

    /// Stream every scenario produces to and subscribes on. In external
    /// mode it must exist in the server's manifest with an audience the
    /// minted token satisfies.
    #[arg(long, global = true, default_value = "loadgen")]
    pub stream: String,

    /// Measurement window, in seconds.
    #[arg(long, global = true, default_value_t = 10)]
    pub duration: u64,

    /// Emit the result as a single JSON object instead of a human report.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub scenario: Scenario,
}

#[derive(Debug, Subcommand)]
pub enum Scenario {
    /// Saturate the dispatch path: hold N subscribers, push events as
    /// fast as a pool of producers can, and report the ingest ceiling.
    Throughput {
        /// Concurrent subscribers held on the measured stream.
        #[arg(long, default_value_t = 50)]
        subscribers: usize,
        /// Concurrent producer tasks issuing pushes.
        #[arg(long, default_value_t = 8)]
        producers: usize,
    },
    /// Measure push→deliver latency under a paced producer.
    Latency {
        /// Concurrent subscribers; each one contributes latency samples.
        #[arg(long, default_value_t = 10)]
        subscribers: usize,
        /// Producer push interval, in milliseconds (one event per tick).
        #[arg(long, default_value_t = 5)]
        interval_ms: u64,
    },
    /// Open and hold N connections; report the sustained active counts.
    Connections {
        /// Connections to open and hold for the measurement window.
        #[arg(long, default_value_t = 500)]
        count: usize,
    },
}
