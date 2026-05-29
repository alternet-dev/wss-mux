//! End-to-end behaviour of the SSE read endpoint (`GET /events/:stream`).
//!
//! Auth supports two paths:
//!
//! - **Shared bearer** via `WSS_MUX_READ_AUTH_TOKEN` — full read across
//!   streams, no audience check.
//! - **JWT** — the connection's principals must intersect the stream's
//!   `subscribe` audience.
//!
//! Both can be configured at once; either path alone admits.

use std::path::Path;
use std::time::Duration;

use reqwest::header::ACCEPT;
use serde_json::Value;
use wss_mux::config::Config;
use wss_mux::manifest::Manifest;
use wss_mux::server::AppState;

use crate::common::{poll_until, sign_token, spawn_server, PUSH_TOKEN, SIGNING_KEY};

const MANIFEST: &str = r#"
version: 1
streams:
  - stream: chat
    subscribe: [role:member]
  - stream: ops
    subscribe: [role:operator]
"#;

fn state_with_read_token(token: Option<&str>) -> AppState {
    let mut c = Config::new(
        PUSH_TOKEN.into(),
        wss_mux::config::HandshakeKeyConfig {
            hs256_secret: Some(SIGNING_KEY.into()),
            ed25519_public_pem: None,
        },
        "t.yaml".into(),
    );
    c.inbound_rate_per_sec = 0;
    c.inbound_burst = 0;
    c.read_auth_token = token.map(str::to_string);
    let state = AppState::new(c);
    state.set_manifest(Manifest::from_str(MANIFEST, Path::new("t.yaml")).expect("manifest"));
    state
}

/// Open an SSE consumer + poll until the server has registered the
/// subscription (so the next push is guaranteed to be observed).
async fn open_sse_and_wait_for_binding(
    addr: std::net::SocketAddr,
    state: &AppState,
    path: &str,
    bearer: &str,
    stream_for_binding: &str,
) -> reqwest::Response {
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}{path}"))
        .header(ACCEPT, "text/event-stream")
        .header("Authorization", format!("Bearer {bearer}"))
        .send()
        .await
        .expect("sse connect");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "text/event-stream"
    );
    // Wait for the subscription to register before returning.
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count(stream_for_binding) >= 1
        })
        .await
    );
    resp
}

/// Read one `data:` line worth of JSON from the SSE response body and
/// return the parsed value. Comments/keep-alives and other SSE lines
/// are skipped.
async fn next_data_event(resp: &mut reqwest::Response) -> Value {
    let timeout = Duration::from_secs(2);
    let deadline = std::time::Instant::now() + timeout;
    let mut buf = String::new();
    loop {
        if std::time::Instant::now() > deadline {
            panic!("timed out waiting for SSE data event; buffered: {buf:?}");
        }
        let chunk = tokio::time::timeout(Duration::from_millis(200), resp.chunk()).await;
        let chunk = match chunk {
            Ok(Ok(Some(bytes))) => bytes,
            Ok(Ok(None)) => panic!("SSE stream ended before data event; buffered: {buf:?}"),
            Ok(Err(e)) => panic!("SSE chunk error: {e}"),
            Err(_) => continue,
        };
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(end) = buf.find("\n\n") {
            let (frame, rest) = buf.split_at(end);
            let frame = frame.to_string();
            buf = rest[2..].to_string();
            for line in frame.lines() {
                if let Some(json) = line.strip_prefix("data: ") {
                    return serde_json::from_str(json)
                        .unwrap_or_else(|e| panic!("bad SSE JSON {json:?}: {e}"));
                }
            }
        }
    }
}

async fn push_event(addr: std::net::SocketAddr, body: Value) -> reqwest::StatusCode {
    reqwest::Client::new()
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&body)
        .send()
        .await
        .expect("push")
        .status()
}

#[tokio::test]
async fn shared_bearer_consumer_receives_pushed_event() {
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state.clone()).await;

    let mut sse =
        open_sse_and_wait_for_binding(addr, &state, "/events/chat", "read-secret", "chat").await;

    let status = push_event(
        addr,
        serde_json::json!({"stream": "chat", "key": "room-1", "payload": {"text": "hi"}}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::NO_CONTENT);

    let event = next_data_event(&mut sse).await;
    assert_eq!(event.get("type").and_then(Value::as_str), Some("event"));
    assert_eq!(event.get("stream").and_then(Value::as_str), Some("chat"));
    assert_eq!(event.get("key").and_then(Value::as_str), Some("room-1"));
    // SSE consumers see `id: ""` — there is no client-supplied sub id.
    assert_eq!(event.get("id").and_then(Value::as_str), Some(""));
}

#[tokio::test]
async fn key_query_param_narrows_to_one_key() {
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state.clone()).await;

    let mut sse = open_sse_and_wait_for_binding(
        addr,
        &state,
        "/events/chat?key=room-7",
        "read-secret",
        "chat",
    )
    .await;

    // Unrelated key push — must NOT be observed.
    let _ = push_event(
        addr,
        serde_json::json!({"stream": "chat", "key": "room-1", "payload": {"text": "skip me"}}),
    )
    .await;
    // Matching key push — must arrive.
    let _ = push_event(
        addr,
        serde_json::json!({"stream": "chat", "key": "room-7", "payload": {"text": "deliver me"}}),
    )
    .await;

    let event = next_data_event(&mut sse).await;
    assert_eq!(event.get("key").and_then(Value::as_str), Some("room-7"));
    assert_eq!(
        event
            .get("payload")
            .and_then(|p| p.get("text"))
            .and_then(Value::as_str),
        Some("deliver me")
    );
}

#[tokio::test]
async fn jwt_admit_when_principals_intersect_subscribe_audience() {
    // No shared bearer set ⇒ only JWT is accepted.
    let state = state_with_read_token(None);
    let addr = spawn_server(state.clone()).await;

    let token = sign_token(&["role:member"]);
    let mut sse = open_sse_and_wait_for_binding(addr, &state, "/events/chat", &token, "chat").await;

    let _ = push_event(
        addr,
        serde_json::json!({"stream": "chat", "payload": {"x": 1}}),
    )
    .await;
    let event = next_data_event(&mut sse).await;
    assert_eq!(event.get("stream").and_then(Value::as_str), Some("chat"));
}

#[tokio::test]
async fn jwt_forbidden_when_principals_dont_intersect() {
    let state = state_with_read_token(None);
    let addr = spawn_server(state).await;

    // `ops` requires role:operator; we authenticate as role:member.
    let token = sign_token(&["role:member"]);
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/events/ops"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("sse");
    assert_eq!(resp.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn no_bearer_is_401() {
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state).await;

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/events/chat"))
        .send()
        .await
        .expect("sse");
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_shared_bearer_falls_through_to_jwt_and_401s() {
    // Shared bearer is configured but the request supplies a different
    // value. Since the value is neither the shared bearer nor a valid
    // JWT, it must 401 (not silently fall through).
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state).await;

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/events/chat"))
        .header("Authorization", "Bearer not-the-right-token")
        .send()
        .await
        .expect("sse");
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unknown_stream_is_404() {
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state).await;

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/events/no_such_stream"))
        .header("Authorization", "Bearer read-secret")
        .send()
        .await
        .expect("sse");
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn dropping_the_response_unregisters_the_subscription() {
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state.clone()).await;

    let resp =
        open_sse_and_wait_for_binding(addr, &state, "/events/chat", "read-secret", "chat").await;
    assert_eq!(state.registry().binding_count("chat"), 1);

    // Drop the response → the SSE stream's `Drop` runs the cleanup.
    drop(resp);

    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat") == 0
        })
        .await,
        "binding should be cleared shortly after the consumer disconnects"
    );
}

#[tokio::test]
async fn shared_bearer_takes_priority_over_jwt() {
    // Both configured: a request that uses the shared bearer is admitted
    // *without* a JWT-style principals check. The chat stream's audience
    // would otherwise need role:member — but the shared-bearer path is
    // unrestricted by design.
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state.clone()).await;

    let mut sse =
        open_sse_and_wait_for_binding(addr, &state, "/events/ops", "read-secret", "ops").await;

    let _ = push_event(
        addr,
        serde_json::json!({"stream": "ops", "payload": {"event": "deploy"}}),
    )
    .await;
    let event = next_data_event(&mut sse).await;
    assert_eq!(event.get("stream").and_then(Value::as_str), Some("ops"));
    // No need to assert the shared bearer's lack of audience check
    // beyond reaching this point; if it had been checked, the GET
    // would have already returned 403 because the shared bearer
    // carries no principals.
    drop(sse);
}

// Note on backpressure overflow: forcing a per-sub overflow against an
// SSE consumer requires the dispatch path to outrun the consumer in
// real wall-clock time. The unit-level backpressure semantics are
// already covered by the dispatcher's own tests (per-sub overflow is
// the same code path whether the consumer is WS or SSE). A separate
// stress test would be the right place to wire in an end-to-end
// overflow assertion against the SSE response; we keep this integration
// suite focused on the request/response and auth surface.

// ---------- multi-consumer behaviour ----------
//
// Each SSE GET registers its own `(conn_id, "sse")` binding in the
// registry; the dispatcher iterates all matching bindings and writes
// to each per-sub channel independently. These tests pin that
// invariant so a future change can't quietly collapse two consumers
// onto a shared channel.

#[tokio::test]
async fn two_sse_consumers_on_the_same_stream_both_receive_each_event() {
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state.clone()).await;

    let mut a =
        open_sse_and_wait_for_binding(addr, &state, "/events/chat", "read-secret", "chat").await;
    // Wait for *two* bindings before pushing — the second connect races
    // against the first's registration completion.
    let mut b = reqwest::Client::new()
        .get(format!("http://{addr}/events/chat"))
        .header(ACCEPT, "text/event-stream")
        .header("Authorization", "Bearer read-secret")
        .send()
        .await
        .expect("sse connect b");
    assert_eq!(b.status(), reqwest::StatusCode::OK);
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat") >= 2
        })
        .await
    );

    let _ = push_event(
        addr,
        serde_json::json!({"stream": "chat", "key": "room-1", "payload": {"text": "two-fer"}}),
    )
    .await;

    let event_a = next_data_event(&mut a).await;
    let event_b = next_data_event(&mut b).await;
    for ev in [&event_a, &event_b] {
        assert_eq!(ev.get("stream").and_then(Value::as_str), Some("chat"));
        assert_eq!(ev.get("key").and_then(Value::as_str), Some("room-1"));
        assert_eq!(
            ev.get("payload")
                .and_then(|p| p.get("text"))
                .and_then(Value::as_str),
            Some("two-fer")
        );
    }
}

#[tokio::test]
async fn sse_consumers_with_different_keys_receive_only_their_match() {
    // Two consumers on the same stream with different `?key=` filters.
    // The registry's per-binding `key.is_none() || key == event.key`
    // check means each one observes only its matching events.
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state.clone()).await;

    let mut a = open_sse_and_wait_for_binding(
        addr,
        &state,
        "/events/chat?key=room-1",
        "read-secret",
        "chat",
    )
    .await;
    let mut b = reqwest::Client::new()
        .get(format!("http://{addr}/events/chat?key=room-2"))
        .header(ACCEPT, "text/event-stream")
        .header("Authorization", "Bearer read-secret")
        .send()
        .await
        .expect("sse connect b");
    assert_eq!(b.status(), reqwest::StatusCode::OK);
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat") >= 2
        })
        .await
    );

    // Two pushes, one per key. Each consumer should see exactly one.
    let _ = push_event(
        addr,
        serde_json::json!({"stream": "chat", "key": "room-1", "payload": {"who": "a"}}),
    )
    .await;
    let _ = push_event(
        addr,
        serde_json::json!({"stream": "chat", "key": "room-2", "payload": {"who": "b"}}),
    )
    .await;

    let event_a = next_data_event(&mut a).await;
    assert_eq!(event_a.get("key").and_then(Value::as_str), Some("room-1"));
    assert_eq!(
        event_a
            .get("payload")
            .and_then(|p| p.get("who"))
            .and_then(Value::as_str),
        Some("a")
    );

    let event_b = next_data_event(&mut b).await;
    assert_eq!(event_b.get("key").and_then(Value::as_str), Some("room-2"));
    assert_eq!(
        event_b
            .get("payload")
            .and_then(|p| p.get("who"))
            .and_then(Value::as_str),
        Some("b")
    );
}

#[tokio::test]
async fn dropping_one_sse_consumer_does_not_affect_the_other() {
    // Per-consumer cleanup must not disturb peers. We open two
    // consumers, drop one, and confirm the second still receives a
    // subsequent push (and the registry has exactly one binding left).
    let state = state_with_read_token(Some("read-secret"));
    let addr = spawn_server(state.clone()).await;

    let a =
        open_sse_and_wait_for_binding(addr, &state, "/events/chat", "read-secret", "chat").await;
    let mut b = reqwest::Client::new()
        .get(format!("http://{addr}/events/chat"))
        .header(ACCEPT, "text/event-stream")
        .header("Authorization", "Bearer read-secret")
        .send()
        .await
        .expect("sse connect b");
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat") >= 2
        })
        .await
    );

    // Drop A and wait for the registry to reflect that.
    drop(a);
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat") == 1
        })
        .await
    );

    // B still receives the next push.
    let _ = push_event(
        addr,
        serde_json::json!({"stream": "chat", "payload": {"after_a_dropped": true}}),
    )
    .await;
    let event = next_data_event(&mut b).await;
    assert_eq!(
        event
            .get("payload")
            .and_then(|p| p.get("after_a_dropped"))
            .and_then(Value::as_bool),
        Some(true)
    );
}
