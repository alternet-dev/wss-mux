use axum::routing::get;
use axum::Router;

pub fn build_app() -> Router {
    Router::new().route("/healthz", get(healthz))
}

async fn healthz() -> &'static str {
    "ok"
}
