pub mod http;
pub mod metrics;
pub mod ws;

use self::metrics::Metrics;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use dashmap::DashMap;
use tokio::sync::watch;

use crate::auth::HandshakeVerifier;
use crate::config::Config;
use crate::connection::{ConnId, OutboundTx};
use crate::manifest::{Manifest, ManifestError};
use crate::peers::PeerUrl;
use crate::registry::Registry;
use crate::subqueue::SubQueues;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    config: Config,
    // Swappable so SIGHUP can hot-reload. `watch` gives both a cheap
    // borrow() read on the dispatch/subscribe path and a change signal
    // connections subscribe to for re-validation.
    manifest_tx: watch::Sender<Option<Arc<Manifest>>>,
    // Discovered peer set, refreshed by the discovery task. Same
    // watch pattern as the manifest: cheap borrow() on the relay path,
    // single writer (the refresh task / tests via set_peers).
    peers_tx: watch::Sender<Arc<Vec<PeerUrl>>>,
    // Built once at startup from PeerConfig (timeout + optional TLS).
    relay_client: reqwest::Client,
    // Built once at startup from the configured handshake key(s); a
    // bad key fails startup loudly rather than rejecting every
    // connection at runtime.
    handshake_verifier: HandshakeVerifier,
    registry: Registry,
    // Per-(connection, subscription) in-flight accounting. The
    // dispatcher reserves a slot before enqueuing; the writer releases
    // once written. A subscription that cannot reserve has overflowed
    // and is dropped on its own.
    sub_queues: SubQueues,
    // The per-connection outbound channel. v0.4 overflow is
    // per-subscription and keep-open, so there is no longer a
    // connection-level abort signal here.
    connections: DashMap<ConnId, OutboundTx>,
    next_conn_id: AtomicU64,
    metrics: Metrics,
}

impl AppState {
    /// Fallible constructor: builds the relay HTTP client from the peer
    /// TLS config, so a bad CA/cert path fails startup loudly rather
    /// than silently degrading.
    pub fn try_new(config: Config) -> anyhow::Result<Self> {
        let relay_client = crate::peers::build_relay_client(&config.peers)?;
        let handshake_verifier = HandshakeVerifier::new(
            config.handshake_keys.hs256_secret.as_deref(),
            config.handshake_keys.ed25519_public_pem.as_deref(),
        )?;
        let (manifest_tx, _) = watch::channel(None);
        let (peers_tx, _) = watch::channel(Arc::new(Vec::new()));
        Ok(Self {
            inner: Arc::new(Inner {
                config,
                manifest_tx,
                peers_tx,
                relay_client,
                handshake_verifier,
                registry: Registry::new(),
                sub_queues: SubQueues::new(),
                connections: DashMap::new(),
                next_conn_id: AtomicU64::new(1),
                metrics: Metrics::default(),
            }),
        })
    }

    /// Infallible constructor for tests and call sites with a known-good
    /// (default/plaintext) peer config.
    pub fn new(config: Config) -> Self {
        Self::try_new(config).expect("relay client build (default peer config is always valid)")
    }

    pub fn metrics(&self) -> &Metrics {
        &self.inner.metrics
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    /// Current manifest snapshot. The returned `Arc` is independent of any
    /// concurrent hot-reload, so a holder (e.g. the dispatcher) can keep
    /// using it across a swap.
    pub fn manifest(&self) -> Option<Arc<Manifest>> {
        self.inner.manifest_tx.borrow().clone()
    }

    /// Install/replace the manifest. Called once at startup and again on
    /// each successful SIGHUP reload. `send_replace` never errors even with
    /// no receivers.
    pub fn set_manifest(&self, manifest: Manifest) {
        self.inner
            .manifest_tx
            .send_replace(Some(Arc::new(manifest)));
    }

    /// A receiver connections `select!` on to re-validate their
    /// subscriptions when the manifest changes.
    pub fn subscribe_manifest(&self) -> watch::Receiver<Option<Arc<Manifest>>> {
        self.inner.manifest_tx.subscribe()
    }

    /// Current peer-set snapshot. Cheap `Arc` clone, independent of a
    /// concurrent discovery refresh.
    pub fn peers(&self) -> Arc<Vec<PeerUrl>> {
        self.inner.peers_tx.borrow().clone()
    }

    /// Replace the peer set (discovery refresh task, or tests). Also
    /// publishes the `wss_mux_peers_known` gauge so it tracks the live
    /// set from a single place.
    pub fn set_peers(&self, peers: Vec<PeerUrl>) {
        self.inner.metrics.peers_known.set(peers.len() as i64);
        self.inner.peers_tx.send_replace(Arc::new(peers));
    }

    /// The shared relay HTTP client (per-relay timeout + optional TLS).
    pub fn relay_client(&self) -> &reqwest::Client {
        &self.inner.relay_client
    }

    /// Load the manifest from `path` and swap it in. Returns the stream
    /// count on success. On failure the previous manifest is retained
    /// (the error is returned for the caller to log/meter).
    pub fn reload_manifest(&self, path: &Path) -> Result<usize, ManifestError> {
        let manifest = Manifest::load(path)?;
        let count = manifest.streams.len();
        self.set_manifest(manifest);
        Ok(count)
    }

    pub fn registry(&self) -> &Registry {
        &self.inner.registry
    }

    /// Shared per-subscription in-flight accounting.
    pub fn sub_queues(&self) -> &SubQueues {
        &self.inner.sub_queues
    }

    /// The handshake-token verifier built at startup from the
    /// configured HS256 / Ed25519 key(s).
    pub fn handshake_verifier(&self) -> &HandshakeVerifier {
        &self.inner.handshake_verifier
    }

    pub fn next_conn_id(&self) -> ConnId {
        self.inner.next_conn_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn register_connection(&self, conn_id: ConnId, data_tx: OutboundTx) {
        self.inner.connections.insert(conn_id, data_tx);
    }

    pub fn unregister_connection(&self, conn_id: ConnId) {
        self.inner.connections.remove(&conn_id);
    }

    pub fn sender(&self, conn_id: ConnId) -> Option<OutboundTx> {
        self.inner.connections.get(&conn_id).map(|e| e.clone())
    }
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(http::metrics))
        .route("/v1/events", post(http::push_event))
        .route("/v1/events/batch", post(http::push_batch))
        .route("/internal/v1/relay", post(http::relay_receive))
        .route("/v1/stream", get(ws::ws_handler))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.manifest().is_some() {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            push_auth_token: "t".into(),
            handshake_keys: crate::config::HandshakeKeyConfig {
                hs256_secret: Some("k".into()),
                ed25519_public_pem: None,
            },
            manifest_path: "p".into(),
            queue_depth: 8,
            envelope_stream_path: "stream".into(),
            envelope_key_path: "key".into(),
            envelope_payload_path: "payload".into(),
            inbound_rate_per_sec: 0,
            inbound_burst: 0,
            peers: crate::config::PeerConfig::default(),
        }
    }

    fn manifest(yaml: &str) -> Manifest {
        Manifest::from_str(yaml, Path::new("test.yaml")).expect("valid manifest")
    }

    #[test]
    fn app_state_exposes_shared_sub_queues() {
        let state = AppState::new(cfg());
        assert!(state.sub_queues().try_reserve(1, "s1", 1));
        assert!(!state.sub_queues().try_reserve(1, "s1", 1));
        assert_eq!(state.sub_queues().in_flight(1, "s1"), 1);
    }

    #[test]
    fn manifest_starts_unset_then_set_and_is_swappable() {
        let state = AppState::new(cfg());
        assert!(state.manifest().is_none());

        state.set_manifest(manifest(
            "version: 1\nstreams:\n  - stream: a\n    audience: [role:x]\n",
        ));
        assert!(state.manifest().expect("set").stream("a").is_some());

        // A second set swaps — the old OnceLock would have errored here.
        state.set_manifest(manifest(
            "version: 1\nstreams:\n  - stream: b\n    audience: [role:y]\n",
        ));
        let m = state.manifest().expect("swapped");
        assert!(m.stream("a").is_none());
        assert!(m.stream("b").is_some());
    }

    #[test]
    fn reload_manifest_swaps_from_file_and_retains_old_on_error() {
        let state = AppState::new(cfg());
        let path = std::env::temp_dir().join(format!(
            "wss-mux-reload-{}-{}.yaml",
            std::process::id(),
            std::thread::current().name().unwrap_or("t")
        ));
        std::fs::write(
            &path,
            "version: 1\nstreams:\n  - stream: live\n    audience: [role:x]\n",
        )
        .expect("write fixture");

        let count = state.reload_manifest(&path).expect("reload ok");
        assert_eq!(count, 1);
        assert!(state.manifest().expect("loaded").stream("live").is_some());

        // Missing file: error surfaced, previous manifest retained.
        let missing = std::env::temp_dir().join("wss-mux-reload-absent-zzzz.yaml");
        assert!(state.reload_manifest(&missing).is_err());
        assert!(state.manifest().expect("retained").stream("live").is_some());

        let _ = std::fs::remove_file(&path);
    }
}
