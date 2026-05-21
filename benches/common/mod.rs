//! Shared fixtures for the criterion microbenches. Each bench file
//! pulls this in via `#[path = "common/mod.rs"] mod common;`. Because
//! every `[[bench]]` is its own crate, an includer that uses only some
//! of these helpers triggers `dead_code` — so the `mod common;` line
//! carries `#[allow(dead_code)]`.

use serde_json::json;

use wss_mux::config::{Config, HandshakeKeyConfig};
use wss_mux::connection::{outbound_channel, OutboundRx};
use wss_mux::envelope::EventEnvelope;
use wss_mux::registry::Registry;
use wss_mux::server::AppState;

/// A minimal `Config` for benchmarks — `Config::new` defaults every
/// optional knob; only `queue_depth` is varied.
pub fn bench_config(queue_depth: usize) -> Config {
    let mut c = Config::new(
        "bench-token".into(),
        HandshakeKeyConfig {
            hs256_secret: Some("bench-secret".into()),
            ed25519_public_pem: None,
        },
        "bench.yaml".into(),
    );
    c.queue_depth = queue_depth;
    c
}

/// A sample keyless event envelope on `stream`.
pub fn event(stream: &str) -> EventEnvelope {
    EventEnvelope {
        stream: stream.to_string(),
        key: None,
        payload: json!({ "text": "benchmark event payload" }),
    }
}

/// Keeps a per-subscription receiver alive (so `dispatch`'s `try_send`
/// never sees `Closed`) and lets the dispatch bench drain it between
/// timed iterations (so a bounded channel never fills → `dispatch`
/// never takes the registry-mutating overflow path).
pub struct SubHandle(OutboundRx);

impl SubHandle {
    pub fn drain(&mut self) {
        match &mut self.0 {
            OutboundRx::Bounded(rx) => while rx.try_recv().is_ok() {},
            OutboundRx::Unbounded(rx) => while rx.try_recv().is_ok() {},
        }
    }
}

/// Build an `AppState` whose registry holds `n` subscriptions on
/// `stream`, each with a live per-sub channel of capacity `cap`.
/// Returns the state plus the receiver handles — keep them alive for
/// the whole bench and `drain` them between timed `dispatch` calls.
pub fn state_with_subs(
    stream: &str,
    n: usize,
    cap: usize,
    keyed: bool,
) -> (AppState, Vec<SubHandle>) {
    let state = AppState::new(bench_config(cap));
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let conn_id = i as u64;
        let sub_id = format!("s{i}");
        let (tx, rx) = outbound_channel(cap);
        state.register_sub_sender(conn_id, &sub_id, tx, cap);
        let key = keyed.then(|| format!("key-{i}"));
        state.registry().subscribe(stream, conn_id, sub_id, key);
        handles.push(SubHandle(rx));
    }
    (state, handles)
}

/// Build a standalone `Registry` with `n` subscriptions on `stream`.
/// `keyed` ⇒ each subscription carries a distinct key.
pub fn registry_with_subs(stream: &str, n: usize, keyed: bool) -> Registry {
    let registry = Registry::new();
    for i in 0..n {
        let key = keyed.then(|| format!("key-{i}"));
        registry.subscribe(stream, i as u64, format!("s{i}"), key);
    }
    registry
}
