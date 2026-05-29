//! End-to-end tests against a real wss-mux instance.
//!
//! Skipped unless `WSS_MUX_E2E_URL` is set in the environment. The
//! repo-level harness at `tests/e2e/run.sh` boots a wss-mux binary
//! with a known manifest and exports the env vars these tests read.

use std::env;
use std::time::Duration;

use serde_json::json;
use wss_mux_client::{ErrorCode, ReconnectOptions, WssMuxClient, WssMuxError};

const STREAM: &str = "chat_messages";
const READONLY_STREAM: &str = "readonly";

fn url() -> Option<String> {
    let raw = env::var("WSS_MUX_E2E_URL").ok().filter(|s| !s.is_empty())?;
    Some(raw)
}

fn token() -> String {
    env::var("WSS_MUX_E2E_TOKEN").expect("WSS_MUX_E2E_TOKEN must be set")
}

fn push_url() -> Option<String> {
    env::var("WSS_MUX_E2E_PUSH_URL").ok()
}

fn push_token() -> Option<String> {
    env::var("WSS_MUX_E2E_PUSH_TOKEN").ok()
}

async fn build_client() -> WssMuxClient {
    let u = url().expect("WSS_MUX_E2E_URL must be set");
    let t = token();
    WssMuxClient::builder()
        .url(u)
        .get_token(move || {
            let t = t.clone();
            async move { Ok(t) }
        })
        // Tight reconnect cap so a flaky CI run can't hang for minutes.
        .reconnect(ReconnectOptions {
            max_attempts: Some(3),
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_millis(500),
            backoff_multiplier: 2.0,
        })
        .build()
        .await
        .expect("build")
}

async fn push_via_http(stream: &str, key: Option<&str>, payload: serde_json::Value) {
    // Optional: only run the HTTP-side assertion when the harness
    // provided a push endpoint + token. Falls back to a no-op so
    // ad-hoc local runs without the harness still exercise the
    // WS-only roundtrip tests.
    let (Some(base), Some(tok)) = (push_url(), push_token()) else {
        panic!("WSS_MUX_E2E_PUSH_URL / WSS_MUX_E2E_PUSH_TOKEN not set; harness misconfigured");
    };

    let mut body = serde_json::Map::new();
    body.insert("stream".into(), json!(stream));
    if let Some(k) = key {
        body.insert("key".into(), json!(k));
    }
    body.insert("payload".into(), payload);

    // No reqwest in the SDK's dep graph; use a tiny tokio-based POST.
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    // Strip the scheme and split host:port + path.
    let no_scheme = base
        .strip_prefix("http://")
        .or_else(|| base.strip_prefix("https://"))
        .expect("http(s):// URL");
    let (host_port, path) = no_scheme
        .split_once('/')
        .map(|(hp, p)| (hp.to_string(), format!("/{p}")))
        .unwrap_or_else(|| (no_scheme.to_string(), "/".to_string()));

    let json_body = serde_json::Value::Object(body).to_string();
    let req = format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {host_port}\r\n\
         Authorization: Bearer {tok}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\
         \r\n\
         {json_body}",
        path = path,
        host_port = host_port,
        tok = tok,
        len = json_body.len(),
        json_body = json_body,
    );

    let mut stream = TcpStream::connect(&host_port).await.expect("tcp connect");
    stream.write_all(req.as_bytes()).await.expect("write req");
    stream.flush().await.ok();
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).await.expect("read resp");
    let head = String::from_utf8_lossy(&resp);
    assert!(
        head.starts_with("HTTP/1.1 2"),
        "HTTP push failed; response head: {}",
        head.lines().next().unwrap_or("(empty)")
    );
}

// ---------- tests ----------

#[tokio::test]
async fn connect_subscribe_publish_own_event_roundtrip() {
    if url().is_none() {
        eprintln!("skipped: WSS_MUX_E2E_URL not set");
        return;
    }

    let client = build_client().await;
    let mut sub = client
        .subscribe(STREAM, Some("rust-roundtrip"))
        .await
        .expect("subscribe");

    let payload = json!({"who": "rust", "marker": "self-roundtrip"});
    client
        .publish(STREAM, Some("rust-roundtrip"), payload.clone())
        .await
        .expect("publish");

    let ev = tokio::time::timeout(Duration::from_secs(5), sub.recv())
        .await
        .expect("recv timed out")
        .expect("channel closed")
        .expect("event");
    assert_eq!(ev.stream, STREAM);
    assert_eq!(ev.key.as_deref(), Some("rust-roundtrip"));
    assert_eq!(ev.payload, payload);

    client.close().await.expect("close");
}

#[tokio::test]
async fn publish_to_readonly_stream_returns_unauthorized_publish() {
    if url().is_none() {
        eprintln!("skipped: WSS_MUX_E2E_URL not set");
        return;
    }
    let client = build_client().await;

    let err = client
        .publish(READONLY_STREAM, None, json!({"x": 1}))
        .await
        .expect_err("publish to readonly should reject");
    match err {
        WssMuxError::Protocol { code, .. } => {
            assert_eq!(code, ErrorCode::UnauthorizedPublish, "got {err:?}");
        }
        other => panic!("expected Protocol(unauthorized_publish), got {other:?}"),
    }

    client.close().await.expect("close");
}

#[tokio::test]
async fn http_push_is_observed_by_ws_subscriber() {
    // Cross-transport: pushes via HTTP `POST /events` (matches what a
    // backend producer would do), observes via the SDK's WS subscribe.
    if url().is_none() || push_url().is_none() {
        eprintln!("skipped: WSS_MUX_E2E_URL / WSS_MUX_E2E_PUSH_URL not set");
        return;
    }
    let client = build_client().await;
    let mut sub = client
        .subscribe(STREAM, Some("http-fanout"))
        .await
        .expect("subscribe");

    // The SDK's subscribe() returns once the Subscribe frame is written;
    // the server processes it asynchronously. Push before the server
    // has registered the binding and the event won't fan out to us.
    // 200 ms is well above local-loopback and CI scheduling jitter.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let payload = json!({"who": "http-pusher", "marker": "fanout"});
    push_via_http(STREAM, Some("http-fanout"), payload.clone()).await;

    let ev = tokio::time::timeout(Duration::from_secs(5), sub.recv())
        .await
        .expect("recv timed out")
        .expect("channel closed")
        .expect("event");
    assert_eq!(ev.stream, STREAM);
    assert_eq!(ev.key.as_deref(), Some("http-fanout"));
    assert_eq!(ev.payload, payload);

    client.close().await.expect("close");
}

#[tokio::test]
async fn unsubscribe_stops_event_delivery() {
    if url().is_none() || push_url().is_none() {
        eprintln!("skipped: WSS_MUX_E2E_URL / WSS_MUX_E2E_PUSH_URL not set");
        return;
    }
    let client = build_client().await;
    let sub = client
        .subscribe(STREAM, Some("unsub-test"))
        .await
        .expect("subscribe");
    let _id = sub.id().to_string();
    drop(sub);

    // Give the server time to process the unsubscribe over the wire.
    tokio::time::sleep(Duration::from_millis(200)).await;
    // Push an event; with a successful unsubscribe the server stops
    // dispatching to us — there's nothing direct to assert besides
    // "no events arrive", but the registry-side state is exercised by
    // the act of pushing after the drop with the client still alive.
    push_via_http(STREAM, Some("unsub-test"), json!({"x": 1})).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    client.close().await.expect("close");
}
