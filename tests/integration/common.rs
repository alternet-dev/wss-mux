use std::net::SocketAddr;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use wss_mux::auth::Claims;
use wss_mux::config::Config;
use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::manifest::Manifest;
use wss_mux::server::{build_app, AppState};

pub const PUSH_TOKEN: &str = "test-push-token";
pub const SIGNING_KEY: &str = "test-signing-key";

// Deterministic Ed25519 test keypair (PKCS8 / SPKI PEM), matching the
// pair used by the auth unit tests.
pub const ED25519_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEINS+Ri6hNJgwRt84yvchqfGNA8ufVJ/7PlEI7O1RQPWe\n-----END PRIVATE KEY-----\n";
pub const ED25519_PUB_PEM: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAGru6jfUFXaDDOSCuIvObd8KSbVpkQb43iORKVTKZuMw=\n-----END PUBLIC KEY-----\n";

pub type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub fn test_config() -> Config {
    Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        push_auth_token: PUSH_TOKEN.into(),
        handshake_keys: wss_mux::config::HandshakeKeyConfig {
            hs256_secret: Some(SIGNING_KEY.into()),
            ed25519_public_pem: None,
        },
        oidc: None,
        manifest_path: "test.yaml".into(),
        queue_depth: 1024,
        envelope_stream_path: "stream".into(),
        envelope_key_path: "key".into(),
        envelope_payload_path: "payload".into(),
        // Rate limiting off by default so the existing fast-sending tests
        // aren't throttled; the rate-limit suite builds its own config.
        inbound_rate_per_sec: 0,
        inbound_burst: 0,
        // Coalescing off by default in tests ⇒ relay path byte-identical
        // to pre-v0.5; the coalescing suite builds its own config.
        relay_coalesce_ms: 0,
        relay_coalesce_max_events: 1024,
        relay_queue_depth: 1024,
        // Peer relay defaults are inert in-test (no resolvable peers),
        // keeping every existing suite byte-identical to single-instance.
        peers: wss_mux::config::PeerConfig::default(),
    }
}

pub fn test_state() -> AppState {
    AppState::new(test_config())
}

pub fn test_state_with_manifest() -> AppState {
    let state = test_state();
    state.set_manifest(sample_manifest());
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
    sign_token_with_offsets(principals, 0, 300)
}

pub fn sign_token_with_offsets(
    principals: &[&str],
    iat_offset_secs: i64,
    exp_offset_secs: i64,
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let iat = (now + iat_offset_secs).max(0) as u64;
    let exp = (now + exp_offset_secs).max(0) as u64;
    let claims = Claims {
        iss: "test".into(),
        iat,
        exp,
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

/// EdDSA-signed handshake token using [`ED25519_PRIV_PEM`], for servers
/// configured with the matching [`ED25519_PUB_PEM`].
pub fn sign_token_ed25519(principals: &[&str]) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        iss: "test".into(),
        iat: now,
        exp: now + 300,
        sub: "user:test".into(),
        principals: principals.iter().map(|s| (*s).to_string()).collect(),
    };
    encode(
        &Header::new(Algorithm::EdDSA),
        &claims,
        &EncodingKey::from_ed_pem(ED25519_PRIV_PEM.as_bytes()).expect("ed25519 priv pem"),
    )
    .unwrap()
}

// --- WebSocket client helpers --------------------------------------------

pub async fn connect_ws(addr: SocketAddr) -> Ws {
    let url = format!("ws://{addr}/v1/stream");
    let mut req = url.into_client_request().expect("request");
    req.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        "wss-mux.v1".parse().expect("header"),
    );
    let (stream, _) = tokio_tungstenite::connect_async(req)
        .await
        .expect("connect");
    stream
}

pub async fn connect_ws_cbor(addr: SocketAddr) -> Ws {
    let url = format!("ws://{addr}/v1/stream");
    let mut req = url.into_client_request().expect("request");
    req.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        "wss-mux.v1.cbor".parse().expect("header"),
    );
    let (stream, _) = tokio_tungstenite::connect_async(req)
        .await
        .expect("connect");
    stream
}

pub async fn send_text(ws: &mut Ws, text: String) {
    ws.send(WsMessage::Text(text)).await.expect("send");
}

pub async fn send_frame_cbor(ws: &mut Ws, frame: &ClientFrame) {
    let mut buf = Vec::new();
    ciborium::into_writer(frame, &mut buf).expect("cbor encode");
    ws.send(WsMessage::Binary(buf)).await.expect("send");
}

/// Next non-ping/pong message decoded from CBOR into a `ServerFrame`.
pub async fn recv_frame_cbor(ws: &mut Ws) -> ServerFrame {
    match recv_message(ws).await {
        WsMessage::Binary(b) => ciborium::from_reader(&b[..]).expect("cbor decode"),
        other => panic!("expected binary frame, got {other:?}"),
    }
}

pub async fn send_frame(ws: &mut Ws, frame: &ClientFrame) {
    send_text(ws, serde_json::to_string(frame).expect("serialize")).await;
}

/// Reads the next non-ping/pong message from the socket within 2 seconds.
pub async fn recv_message(ws: &mut Ws) -> WsMessage {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
            .await
            .expect("recv timed out")
            .expect("stream ended")
            .expect("recv error");
        match msg {
            WsMessage::Ping(_) | WsMessage::Pong(_) => continue,
            other => return other,
        }
    }
}

/// Expects the next message to be an event frame.
pub async fn recv_event(ws: &mut Ws) -> ServerFrame {
    match recv_message(ws).await {
        WsMessage::Text(t) => serde_json::from_str(t.as_str()).expect("parse frame"),
        other => panic!("expected text frame, got {other:?}"),
    }
}

/// Expects the next message to be an error frame; returns (code, id).
pub async fn recv_error_frame(ws: &mut Ws) -> (String, Option<String>) {
    match recv_message(ws).await {
        WsMessage::Text(t) => match serde_json::from_str(t.as_str()).expect("parse") {
            ServerFrame::Error { code, id, .. } => (code, id),
            other => panic!("expected error frame, got {other:?}"),
        },
        other => panic!("expected text frame, got {other:?}"),
    }
}

/// Expects the next message to be a close frame; returns its code (1005 if
/// the close was sent without a status).
pub async fn recv_close_code(ws: &mut Ws) -> u16 {
    match recv_message(ws).await {
        WsMessage::Close(Some(frame)) => u16::from(frame.code),
        WsMessage::Close(None) => 1005,
        other => panic!("expected close, got {other:?}"),
    }
}

// --- Polling helpers -----------------------------------------------------

/// Polls `cond` every 10ms until it returns true or `timeout` elapses.
/// Returns the final result.
pub async fn poll_until<F: FnMut() -> bool>(timeout: Duration, mut cond: F) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if cond() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    cond()
}

/// Polls `GET /metrics` every 10ms until the body contains `substring` or
/// `timeout` elapses.
pub async fn poll_metrics_for(addr: SocketAddr, substring: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let http = reqwest::Client::new();
    while Instant::now() < deadline {
        if let Ok(resp) = http.get(format!("http://{addr}/metrics")).send().await {
            if let Ok(body) = resp.text().await {
                if body.contains(substring) {
                    return true;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    false
}
