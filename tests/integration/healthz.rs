use wss_mux::server::AppState;

use crate::common::spawn_server;

#[tokio::test]
async fn healthz_returns_200_without_manifest() {
    let addr = spawn_server(AppState::new()).await;
    let resp = reqwest::get(format!("http://{addr}/healthz"))
        .await
        .expect("request");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn healthz_body_is_ok() {
    let addr = spawn_server(AppState::new()).await;
    let body = reqwest::get(format!("http://{addr}/healthz"))
        .await
        .expect("request")
        .text()
        .await
        .expect("body");
    assert_eq!(body, "ok");
}
