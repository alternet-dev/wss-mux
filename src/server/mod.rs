pub mod http;
pub mod metrics;
pub mod relay;
pub mod ws;

use self::metrics::Metrics;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use dashmap::DashMap;
use jsonwebtoken::jwk::JwkSet;
use tokio::sync::watch;

use crate::auth::{HandshakeVerifier, OidcVerifier};
use crate::config::Config;
use crate::connection::{ConnId, OutboundTx};
use crate::manifest::{Manifest, ManifestError};
use crate::peers::PeerUrl;
use crate::registry::Registry;
use crate::server::relay::RelayBatch;

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
    // bad key fails startup loudly. `None` when OIDC is configured
    // (issuer XOR handshake — there is no handshake key to build).
    handshake_verifier: Option<HandshakeVerifier>,
    // Built at startup when OIDC is configured (`Some` iff
    // `handshake_verifier` is `None` — issuer XOR handshake).
    oidc_verifier: Option<OidcVerifier>,
    // Cached OIDC JWKS, refreshed by `spawn_oidc_jwks_refresher`. Same
    // watch pattern as `peers_tx`: cheap borrow() on the validate path,
    // single writer (the refresher / tests via set_jwks). Empty until
    // the first successful fetch (OIDC disabled ⇒ stays empty, inert).
    jwks_tx: watch::Sender<Arc<JwkSet>>,
    registry: Registry,
    // Per-connection control channel: connection-level frames, typed
    // closes, and the keep-open `overflow` error (sent here because the
    // overflowing subscription's own channel is the thing that's full).
    // Unbounded — these are low-volume and must never be shed.
    control: DashMap<ConnId, OutboundTx>,
    // One channel per (connection, subscription) — the (c) model. Value
    // is the sender plus the depth it was created with (`0` ⇒
    // unbounded) for the send-queue-depth metric. The dispatcher sends
    // events here; the writer drains a `StreamMap` of the receivers.
    sub_senders: DashMap<(ConnId, String), (OutboundTx, usize)>,
    next_conn_id: AtomicU64,
    metrics: Metrics,
    // Relay coalescing queue. `Some` only when peer-relay is enabled
    // AND WSS_MUX_RELAY_COALESCE_MS > 0. `accept_events` enqueues here;
    // the supervised flush task drains it. `None` ⇒ coalescing off,
    // relay spawned per push (pre-v0.5 behaviour).
    relay_tx: Option<tokio::sync::mpsc::Sender<RelayBatch>>,
    // Receiver, taken once by the flush-task spawner at startup.
    relay_rx: Mutex<Option<tokio::sync::mpsc::Receiver<RelayBatch>>>,
}

impl AppState {
    /// Fallible constructor: builds the relay HTTP client from the peer
    /// TLS config, so a bad CA/cert path fails startup loudly rather
    /// than silently degrading.
    pub fn try_new(config: Config) -> anyhow::Result<Self> {
        let relay_client = crate::peers::build_relay_client(&config.peers)?;
        // OIDC is mutually exclusive with the handshake key: build the
        // handshake verifier only when OIDC is not configured.
        let handshake_verifier = if config.oidc.is_none() {
            Some(HandshakeVerifier::new(
                config.handshake_keys.hs256_secret.as_deref(),
                config.handshake_keys.ed25519_public_pem.as_deref(),
            )?)
        } else {
            None
        };
        let oidc_verifier = config.oidc.as_ref().map(OidcVerifier::from_config);
        let (manifest_tx, _) = watch::channel(None);
        let (peers_tx, _) = watch::channel(Arc::new(Vec::new()));
        let (jwks_tx, _) = watch::channel(Arc::new(JwkSet { keys: Vec::new() }));
        let (relay_tx, relay_rx) = if config.peers.relay_enabled && config.relay_coalesce_ms > 0 {
            let (tx, rx) = tokio::sync::mpsc::channel(config.relay_queue_depth.max(1));
            (Some(tx), Mutex::new(Some(rx)))
        } else {
            (None, Mutex::new(None))
        };
        Ok(Self {
            inner: Arc::new(Inner {
                config,
                manifest_tx,
                peers_tx,
                relay_client,
                handshake_verifier,
                oidc_verifier,
                jwks_tx,
                registry: Registry::new(),
                control: DashMap::new(),
                sub_senders: DashMap::new(),
                next_conn_id: AtomicU64::new(1),
                metrics: Metrics::default(),
                relay_tx,
                relay_rx,
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

    /// Current OIDC JWKS snapshot. Cheap `Arc` clone, independent of a
    /// concurrent refresh.
    pub fn jwks(&self) -> Arc<JwkSet> {
        self.inner.jwks_tx.borrow().clone()
    }

    /// Replace the cached JWKS (refresher task, or tests). Publishes the
    /// `wss_mux_oidc_jwks_keys` gauge from one place.
    pub fn set_jwks(&self, jwks: JwkSet) {
        self.inner
            .metrics
            .oidc_jwks_keys
            .set(jwks.keys.len() as i64);
        self.inner.jwks_tx.send_replace(Arc::new(jwks));
    }

    /// The shared relay HTTP client (per-relay timeout + optional TLS).
    pub fn relay_client(&self) -> &reqwest::Client {
        &self.inner.relay_client
    }

    /// Relay-coalescing sender (clone per enqueue). `None` ⇒ coalescing
    /// disabled (or peer-relay off) — caller falls back to direct relay.
    pub fn relay_tx(&self) -> Option<tokio::sync::mpsc::Sender<RelayBatch>> {
        self.inner.relay_tx.clone()
    }

    /// Take the relay-coalescing receiver. Returns `Some` at most once
    /// (the flush-task spawner owns it); `None` thereafter or when
    /// coalescing is disabled.
    pub fn take_relay_rx(&self) -> Option<tokio::sync::mpsc::Receiver<RelayBatch>> {
        self.inner.relay_rx.lock().expect("relay_rx mutex").take()
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

    /// The handshake-token verifier built at startup from the
    /// configured HS256 / Ed25519 key(s). `None` when OIDC is
    /// configured instead (mutually exclusive).
    pub fn handshake_verifier(&self) -> Option<&HandshakeVerifier> {
        self.inner.handshake_verifier.as_ref()
    }

    /// The OIDC verifier, built at startup when `WSS_MUX_OIDC_ISSUER`
    /// is set (mutually exclusive with the handshake verifier).
    pub fn oidc_verifier(&self) -> Option<&OidcVerifier> {
        self.inner.oidc_verifier.as_ref()
    }

    pub fn next_conn_id(&self) -> ConnId {
        self.inner.next_conn_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Register the per-connection control sender (connection-level
    /// frames, typed closes, keep-open `overflow` errors).
    pub fn register_control(&self, conn_id: ConnId, tx: OutboundTx) {
        self.inner.control.insert(conn_id, tx);
    }

    /// The control sender for a connection, if it is still registered.
    pub fn control_sender(&self, conn_id: ConnId) -> Option<OutboundTx> {
        self.inner.control.get(&conn_id).map(|e| e.clone())
    }

    /// Register (or replace) the per-subscription event sender. Replace
    /// is used on a SIGHUP `queue_depth` change to swap in a
    /// freshly-sized channel; the old sender is dropped, which the
    /// writer observes and evicts.
    pub fn register_sub_sender(&self, conn_id: ConnId, sub_id: &str, tx: OutboundTx, cap: usize) {
        self.inner
            .sub_senders
            .insert((conn_id, sub_id.to_string()), (tx, cap));
    }

    /// The per-subscription sender and the depth it was built with
    /// (`0` ⇒ unbounded), if the subscription is still live.
    pub fn sub_sender(&self, conn_id: ConnId, sub_id: &str) -> Option<(OutboundTx, usize)> {
        self.inner
            .sub_senders
            .get(&(conn_id, sub_id.to_string()))
            .map(|e| e.clone())
    }

    /// Drop a subscription's sender (unsubscribe / revoke / overflow).
    /// Dropping it ends the receiver, so the writer evicts that
    /// subscription from its `StreamMap` on the next poll.
    pub fn remove_sub_sender(&self, conn_id: ConnId, sub_id: &str) {
        self.inner
            .sub_senders
            .remove(&(conn_id, sub_id.to_string()));
    }

    /// Tear the connection down: drop its control sender and every
    /// per-subscription sender. With all senders gone, the writer's
    /// control channel and `StreamMap` all close and it exits.
    pub fn remove_connection(&self, conn_id: ConnId) {
        self.inner.control.remove(&conn_id);
        self.inner.sub_senders.retain(|(c, _), _| *c != conn_id);
    }
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(http::metrics))
        .route("/events", post(http::push_event))
        .route("/events/batch", post(http::push_batch))
        .route("/internal/relay", post(http::relay_receive))
        .route("/stream", get(ws::ws_handler))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn ready(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.manifest().is_none() {
        return (StatusCode::SERVICE_UNAVAILABLE, "not ready");
    }
    // With OIDC enabled, a never-fetched (empty) JWKS would 401 every
    // client — keep the pod out of rotation until keys are cached.
    if state.config().oidc.is_some() && state.jwks().keys.is_empty() {
        return (StatusCode::SERVICE_UNAVAILABLE, "not ready: oidc jwks");
    }
    (StatusCode::OK, "ready")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        let mut c = Config::new(
            "t".into(),
            crate::config::HandshakeKeyConfig {
                hs256_secret: Some("k".into()),
                ed25519_public_pem: None,
            },
            "p".into(),
        );
        c.queue_depth = 8;
        c.inbound_rate_per_sec = 0;
        c.inbound_burst = 0;
        c
    }

    fn manifest(yaml: &str) -> Manifest {
        Manifest::from_str(yaml, Path::new("test.yaml")).expect("valid manifest")
    }

    #[test]
    fn sub_senders_and_control_register_and_remove() {
        use crate::connection::outbound_channel;
        let state = AppState::new(cfg());
        let (ctl, _ctl_rx) = outbound_channel(0);
        state.register_control(1, ctl);
        assert!(state.control_sender(1).is_some());

        let (tx, _rx) = outbound_channel(8);
        state.register_sub_sender(1, "s1", tx, 8);
        assert!(matches!(state.sub_sender(1, "s1"), Some((_, 8))));

        state.remove_sub_sender(1, "s1");
        assert!(state.sub_sender(1, "s1").is_none());

        // Connection teardown drops control + any remaining sub senders.
        let (tx2, _rx2) = outbound_channel(0);
        state.register_sub_sender(1, "s2", tx2, 0);
        state.remove_connection(1);
        assert!(state.control_sender(1).is_none());
        assert!(state.sub_sender(1, "s2").is_none());
    }

    #[test]
    fn relay_channel_absent_when_coalescing_off() {
        let state = AppState::new(cfg()); // cfg() has relay_coalesce_ms: 0
        assert!(state.relay_tx().is_none());
        assert!(state.take_relay_rx().is_none());
    }

    #[test]
    fn relay_channel_present_when_coalescing_on_and_rx_taken_once() {
        let mut c = cfg();
        c.relay_coalesce_ms = 10;
        // cfg()'s PeerConfig::default() has relay_enabled = true.
        let state = AppState::new(c);
        assert!(state.relay_tx().is_some());
        assert!(state.take_relay_rx().is_some(), "rx available once");
        assert!(state.take_relay_rx().is_none(), "rx taken only once");
    }

    #[test]
    fn manifest_starts_unset_then_set_and_is_swappable() {
        let state = AppState::new(cfg());
        assert!(state.manifest().is_none());

        state.set_manifest(manifest(
            "version: 1\nstreams:\n  - stream: a\n    subscribe: [role:x]\n",
        ));
        assert!(state.manifest().expect("set").stream("a").is_some());

        // A second set swaps — the old OnceLock would have errored here.
        state.set_manifest(manifest(
            "version: 1\nstreams:\n  - stream: b\n    subscribe: [role:y]\n",
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
            "version: 1\nstreams:\n  - stream: live\n    subscribe: [role:x]\n",
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
