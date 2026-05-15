use std::net::SocketAddr;

use tokio::net::TcpListener;
use wss_mux::server::build_app;

async fn spawn_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, build_app()).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn healthz_returns_200() {
    let addr = spawn_server().await;
    let resp = reqwest::get(format!("http://{addr}/healthz"))
        .await
        .expect("request");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn healthz_body_is_ok() {
    let addr = spawn_server().await;
    let body = reqwest::get(format!("http://{addr}/healthz"))
        .await
        .expect("request")
        .text()
        .await
        .expect("body");
    assert_eq!(body, "ok");
}
