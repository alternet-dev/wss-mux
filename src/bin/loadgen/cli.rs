//! Command-line surface for the load/stress harness.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::topology::Topology;

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

    /// Peer instances to spin up alongside the ingress (in-process
    /// only): `0` = single instance, `1` = a single-peer pair, `≥2` = a
    /// multi-peer fleet. With a fleet, producers push to one instance
    /// and subscribers connect to another, so the measured path crosses
    /// a peer-relay hop.
    #[arg(long, global = true, default_value_t = 0)]
    pub peers: usize,

    /// Traffic topology for fleet runs: `broadcast` — subscribers
    /// balanced across the fleet; `concentrated` — all subscribers on
    /// one instance; `keyed` — many small-audience keys, subscribers
    /// scattered. Drives how much cross-instance relay is wasted.
    #[arg(long, global = true, value_enum, default_value = "broadcast")]
    pub topology: Topology,

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
    /// Slow consumer: a subscriber stops reading on a small-queue
    /// stream; confirm it overflows alone while a healthy client
    /// keeps receiving.
    SlowConsumer,
    /// Dead peer: relay to a guaranteed-closed peer; confirm the
    /// failures are metered and the producer is never back-pressured.
    DeadPeer,
    /// Coalesce saturation: burst past a small relay-coalescing queue;
    /// confirm batches drop and meter, with no producer back-pressure.
    CoalesceSaturate,
    /// Reconnect storm: open, drop, and immediately re-establish N
    /// connections in waves; confirm the server recovers each wave.
    ReconnectStorm {
        /// Connections opened (and dropped) per wave.
        #[arg(long, default_value_t = 100)]
        count: usize,
        /// Spread each wave's connects over this window, in
        /// milliseconds; `0` fires the whole wave at once (a pure
        /// thundering herd).
        #[arg(long, default_value_t = 250)]
        jitter_ms: u64,
    },
    /// Payload cap: POST an oversized payload to a capped stream;
    /// confirm a 413 and that the rejection is metered.
    PayloadCap,
    /// Rate limit: flood inbound frames past a low rate limit; confirm
    /// keep-open rate-limited errors and that admission later resumes.
    RateLimit,
    /// Run connection, throughput, and memory sweeps and write one
    /// machine-readable sizing result.
    SizingRun(SizingArgs),
    /// Internal child process used by `sizing-run` to isolate server
    /// resource measurements from the load generator.
    #[command(hide = true)]
    SizingServer {
        #[arg(long, hide = true)]
        ready_file: PathBuf,
    },
}

/// Reproducible sizing-sweep parameters. The defaults are the published
/// methodology; overrides primarily support constrained hosts and smoke tests.
#[derive(Debug, Args)]
pub struct SizingArgs {
    /// Human-readable machine or instance type recorded in the result.
    #[arg(long)]
    pub instance_label: String,

    /// Destination for the complete sizing JSON document.
    #[arg(long)]
    pub out: PathBuf,

    /// First connection count tested by the connection sweep.
    #[arg(long, default_value_t = 100)]
    pub connection_start: usize,

    /// Safety ceiling for the connection sweep.
    #[arg(long, default_value_t = 100_000)]
    pub connection_max: usize,

    /// Fixed event rate used while connection count increases.
    #[arg(long, default_value_t = 100)]
    pub connection_event_rate: u64,

    /// Connection count held during throughput and memory sweeps.
    #[arg(long, default_value_t = 100)]
    pub fixed_connections: usize,

    /// First event rate tested by throughput and memory sweeps.
    #[arg(long, default_value_t = 100)]
    pub event_rate_start: u64,

    /// Safety ceiling for throughput and memory event rates.
    #[arg(long, default_value_t = 1_000_000)]
    pub event_rate_max: u64,

    /// In the memory sweep, every Nth connection stops reading.
    #[arg(long, default_value_t = 10)]
    pub slow_every: usize,

    /// Approximate event payload size used by the memory sweep.
    #[arg(long, default_value_t = 1024)]
    pub memory_payload_bytes: usize,

    /// Percent of available CPU that marks a saturated step.
    #[arg(long, default_value_t = 90, value_parser = clap::value_parser!(u16).range(1..=100))]
    pub cpu_threshold_percent: u16,
}
