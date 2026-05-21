//! Command-line surface for the load/stress harness.

use clap::{Parser, Subcommand};

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
}
