use std::net::SocketAddr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use tokio::net::TcpListener;
use wss_mux::auth::Claims;
use wss_mux::config::Config;
use wss_mux::manifest::Manifest;
use wss_mux::server::{build_app, AppState};

pub const PUSH_TOKEN: &str = "test-push-token";
pub const SIGNING_KEY: &str = "test-signing-key";

pub fn test_config() -> Config {
    Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        push_auth_token: PUSH_TOKEN.into(),
        handshake_signing_key: SIGNING_KEY.into(),
        manifest_path: "test.yaml".into(),
        queue_depth: 1024,
    }
}

pub fn test_state() -> AppState {
    AppState::new(test_config())
}

pub fn test_state_with_manifest() -> AppState {
    let state = test_state();
    state.set_manifest(sample_manifest()).expect("set manifest");
    state
}

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

pub fn sign_token(principals: &[&str]) -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        iss: "test".into(),
        iat: n,
        exp: n + 300,
        sub: "user:test".into(),
        principals: principals.iter().map(|s| (*s).to_string()).collect(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SIGNING_KEY.as_bytes()),
    )
    .unwrap()
}
