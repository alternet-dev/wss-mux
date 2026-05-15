use std::sync::{Arc, OnceLock};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

use crate::manifest::Manifest;

#[derive(Clone, Default)]
pub struct AppState {
    manifest: Arc<OnceLock<Manifest>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn manifest(&self) -> Option<&Manifest> {
        self.manifest.get()
    }

    pub fn set_manifest(&self, manifest: Manifest) -> Result<(), Manifest> {
        self.manifest.set(manifest)
    }
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
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
