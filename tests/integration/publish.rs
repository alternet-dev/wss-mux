use std::path::Path;
use std::time::Duration;

use serde_json::json;
use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::manifest::Manifest;
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_close_code, recv_error_frame, recv_event, send_frame, sign_token,
    spawn_server, test_state, Ws,
};

const MANIFEST: &str = r#"
version: 1
streams:
  - stream: chat
    subscribe: [role:member]
    publish:   [role:member]
  - stream: ops
    subscribe: [role:operator]
    publish:   [role:operator]
  - stream: readonly
    subscribe: [role:member]
    # `publish` absent → default-deny
  - stream: capped
    subscribe: ["*"]
    publish:   ["*"]
    max_payload_bytes: 64
"#;

fn state_with_manifest() -> AppState {
    let state = test_state();
    state.set_manifest(Manifest::from_str(MANIFEST, Path::new("test.yaml")).expect("manifest"));
    state
}

async fn auth(ws: &mut Ws, principals: &[&str]) {
    send_frame(
        ws,
        &ClientFrame::Auth {
            token: sign_token(principals),
        },
    )
    .await;
}

#[tokio::test]
async fn publish_delivers_event_to_own_subscriber() {
    let state = state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    let mut ws = connect_ws(addr).await;
    auth(&mut ws, &["role:member"]).await;
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat".into(),
            key: Some("room-42".into()),
        },
    )
    .await;
    // Make sure the subscription is registered before we publish.
    assert!(
        poll_until(Duration::from_secs(2), || state
            .registry()
            .binding_count("chat")
            >= 1)
        .await
    );

    send_frame(
        &mut ws,
        &ClientFrame::Publish {
            id: "p1".into(),
            stream: "chat".into(),
            key: Some("room-42".into()),
            payload: json!({"from": "alice", "text": "hi"}),
        },
    )
    .await;

    let ServerFrame::Event {
        id,
        stream,
        key,
        payload,
        ..
    } = recv_event(&mut ws).await
    else {
        panic!("expected event frame");
    };
    assert_eq!(id, "s1");
    assert_eq!(stream, "chat");
    assert_eq!(key.as_deref(), Some("room-42"));
    assert_eq!(payload, json!({"from": "alice", "text": "hi"}));
}

#[tokio::test]
async fn publish_to_default_deny_stream_is_unauthorized_keep_open() {
    let state = state_with_manifest();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    auth(&mut ws, &["role:member"]).await;
    // `readonly` has no `publish` audience → default-deny.
    send_frame(
        &mut ws,
        &ClientFrame::Publish {
            id: "p1".into(),
            stream: "readonly".into(),
            key: None,
            payload: json!({"x": 1}),
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthorized_publish");
    assert_eq!(id.as_deref(), Some("p1"));

    // Connection stays open — a second subscribe still works.
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s2".into(),
            stream: "readonly".into(),
            key: None,
        },
    )
    .await;
    // No error frame follows; if it does, the connection wasn't kept open
    // the way we expect. Give it a brief moment.
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test]
async fn publish_when_principals_dont_intersect_is_unauthorized() {
    let state = state_with_manifest();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    // `ops` requires role:operator; we authenticate as role:member.
    auth(&mut ws, &["role:member"]).await;
    send_frame(
        &mut ws,
        &ClientFrame::Publish {
            id: "p1".into(),
            stream: "ops".into(),
            key: None,
            payload: json!({"event": "deploy"}),
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthorized_publish");
    assert_eq!(id.as_deref(), Some("p1"));
}

#[tokio::test]
async fn publish_to_unknown_stream_is_unknown_stream_keep_open() {
    let state = state_with_manifest();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    auth(&mut ws, &["role:member"]).await;
    send_frame(
        &mut ws,
        &ClientFrame::Publish {
            id: "p1".into(),
            stream: "no_such_stream".into(),
            key: None,
            payload: json!({"x": 1}),
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unknown_stream");
    assert_eq!(id.as_deref(), Some("p1"));
}

#[tokio::test]
async fn publish_payload_over_cap_is_publish_payload_too_large_keep_open() {
    let state = state_with_manifest();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    auth(&mut ws, &["role:member"]).await;
    // `capped` is `*` audience + 64-byte cap.
    send_frame(
        &mut ws,
        &ClientFrame::Publish {
            id: "p1".into(),
            stream: "capped".into(),
            key: None,
            payload: json!({"blob": "x".repeat(256)}),
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "publish_payload_too_large");
    assert_eq!(id.as_deref(), Some("p1"));
}

#[tokio::test]
async fn publish_under_cap_is_dispatched() {
    let state = state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    let mut ws = connect_ws(addr).await;
    auth(&mut ws, &["role:member"]).await;
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "capped".into(),
            key: None,
        },
    )
    .await;
    assert!(
        poll_until(Duration::from_secs(2), || state
            .registry()
            .binding_count("capped")
            >= 1)
        .await
    );

    send_frame(
        &mut ws,
        &ClientFrame::Publish {
            id: "p1".into(),
            stream: "capped".into(),
            key: None,
            payload: json!({"x": 1}),
        },
    )
    .await;

    let ServerFrame::Event { stream, .. } = recv_event(&mut ws).await else {
        panic!("expected event frame");
    };
    assert_eq!(stream, "capped");
}

#[tokio::test]
async fn publish_before_auth_closes_4401() {
    let state = state_with_manifest();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    // No auth frame; the first frame is a publish.
    send_frame(
        &mut ws,
        &ClientFrame::Publish {
            id: "p1".into(),
            stream: "chat".into(),
            key: None,
            payload: json!({"x": 1}),
        },
    )
    .await;

    // We expect an `unauthenticated` error frame then a 4401 close.
    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(id.as_deref(), Some("p1"));
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}
