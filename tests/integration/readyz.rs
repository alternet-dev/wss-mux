use wss_mux::server::AppState;

use crate::common::{sample_manifest, spawn_server};

#[tokio::test]
async fn readyz_is_503_before_manifest() {
    let addr = spawn_server(AppState::new()).await;
    let resp = reqwest::get(format!("http://{addr}/readyz"))
        .await
        .expect("request");
    assert_eq!(resp.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    let body = resp.text().await.expect("body");
    assert_eq!(body, "not ready");
}

#[tokio::test]
async fn readyz_is_200_after_manifest() {
    let state = AppState::new();
    state.set_manifest(sample_manifest()).expect("set");
    let addr = spawn_server(state).await;
    let resp = reqwest::get(format!("http://{addr}/readyz"))
        .await
        .expect("request");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.expect("body");
    assert_eq!(body, "ready");
}
