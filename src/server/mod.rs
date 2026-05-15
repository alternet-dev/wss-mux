pub mod http;
pub mod ws;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::connection::ConnId;
use crate::envelope::ServerFrame;
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
    connections: DashMap<ConnId, mpsc::Sender<ServerFrame>>,
    next_conn_id: AtomicU64,
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
            }),
        }
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

    pub fn register_connection(&self, conn_id: ConnId, sender: mpsc::Sender<ServerFrame>) {
        self.inner.connections.insert(conn_id, sender);
    }

    pub fn unregister_connection(&self, conn_id: ConnId) {
        self.inner.connections.remove(&conn_id);
    }

    pub fn sender(&self, conn_id: ConnId) -> Option<mpsc::Sender<ServerFrame>> {
        self.inner.connections.get(&conn_id).map(|e| e.clone())
    }
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/events", post(http::push_event))
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
