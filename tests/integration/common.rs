use std::net::SocketAddr;
use std::path::Path;

use tokio::net::TcpListener;
use wss_mux::manifest::Manifest;
use wss_mux::server::{build_app, AppState};

pub async fn spawn_server(state: AppState) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, build_app(state)).await.unwrap();
    });
    addr
}

pub fn sample_manifest() -> Manifest {
    let yaml = r#"
version: 1
streams:
  - stream: chat_messages
    audience: [role:member]
"#;
    Manifest::from_str(yaml, Path::new("test.yaml")).expect("valid sample manifest")
}
