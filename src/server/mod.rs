pub mod http;
pub mod metrics;
pub mod ws;

use self::metrics::Metrics;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use dashmap::DashMap;
use tokio::sync::{mpsc, watch};

use crate::config::Config;
use crate::connection::{ConnId, Outbound};
use crate::manifest::Manifest;
use crate::registry::Registry;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    config: Config,
    manifest: OnceLock<Manifest>,
    registry: Registry,
    connections: DashMap<ConnId, ConnectionHandle>,
    next_conn_id: AtomicU64,
    metrics: Metrics,
}

/// Per-connection plumbing. The data channel carries normal outbound frames
/// from the dispatcher and reader. The abort channel is a watch of `false`
/// transitioning to `true` exactly once, signaling the reader and writer to
/// short-circuit out of their loops for an overflow close.
struct ConnectionHandle {
    data_tx: mpsc::Sender<Outbound>,
    abort_tx: watch::Sender<bool>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                manifest: OnceLock::new(),
                registry: Registry::new(),
                connections: DashMap::new(),
                next_conn_id: AtomicU64::new(1),
                metrics: Metrics::default(),
            }),
        }
    }

    pub fn metrics(&self) -> &Metrics {
        &self.inner.metrics
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn manifest(&self) -> Option<&Manifest> {
        self.inner.manifest.get()
    }

    pub fn set_manifest(&self, manifest: Manifest) -> Result<(), Manifest> {
        self.inner.manifest.set(manifest)
    }

    pub fn registry(&self) -> &Registry {
        &self.inner.registry
    }

    pub fn next_conn_id(&self) -> ConnId {
        self.inner.next_conn_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn register_connection(
        &self,
        conn_id: ConnId,
        data_tx: mpsc::Sender<Outbound>,
        abort_tx: watch::Sender<bool>,
    ) {
        self.inner
            .connections
            .insert(conn_id, ConnectionHandle { data_tx, abort_tx });
    }

    pub fn unregister_connection(&self, conn_id: ConnId) {
        self.inner.connections.remove(&conn_id);
    }

    pub fn sender(&self, conn_id: ConnId) -> Option<mpsc::Sender<Outbound>> {
        self.inner
            .connections
            .get(&conn_id)
            .map(|e| e.data_tx.clone())
    }

    /// Atomically remove the connection from the registry of senders and
    /// flip its abort signal to `true`. Idempotent — a second call for the
    /// same conn_id is a no-op.
    pub fn trigger_overflow(&self, conn_id: ConnId) {
        if let Some((_, handle)) = self.inner.connections.remove(&conn_id) {
            let _ = handle.abort_tx.send(true);
        }
    }
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(http::metrics))
        .route("/v1/events", post(http::push_event))
        .route("/v1/events/batch", post(http::push_batch))
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
