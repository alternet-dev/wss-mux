use std::path::Path;
use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::manifest::Manifest;

use crate::common::{
    connect_ws, poll_until, recv_error_frame, recv_event, sample_manifest, send_frame, sign_token,
    spawn_server, test_state_with_manifest, PUSH_TOKEN,
};

fn manifest(yaml: &str) -> Manifest {
    Manifest::from_str(yaml, Path::new("test.yaml")).expect("valid manifest")
}

async fn auth_and_subscribe(addr: std::net::SocketAddr) -> crate::common::Ws {
    let mut ws = connect_ws(addr).await;
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;
    ws
}

#[tokio::test]
async fn reload_dropping_a_stream_revokes_its_subscriptions() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;
    let mut ws = auth_and_subscribe(addr).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await
    );

    // Hot-reload to a manifest WITHOUT chat_messages.
    state.set_manifest(manifest(
        "version: 1\nstreams:\n  - stream: other\n    audience: [role:member]\n",
    ));

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unknown_stream");
    assert_eq!(id.as_deref(), Some("s1"));
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") == 0
        })
        .await,
        "revoked binding not removed from registry"
    );

    // Connection stays open: a valid subscribe to a now-present stream works.
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s2".into(),
            stream: "other".into(),
            key: None,
        },
    )
    .await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("other") >= 1
        })
        .await,
        "connection should still accept subscribes after a revocation"
    );
}

#[tokio::test]
async fn reload_tightening_audience_revokes_now_unauthorized_subs() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;
    let mut ws = auth_and_subscribe(addr).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await
    );

    // chat_messages now requires role:operator; the connection only has role:member.
    state.set_manifest(manifest(
        "version: 1\nstreams:\n  - stream: chat_messages\n    audience: [role:operator]\n",
    ));

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthorized_subscribe");
    assert_eq!(id.as_deref(), Some("s1"));
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") == 0
        })
        .await
    );
}

#[tokio::test]
async fn reload_with_same_streams_keeps_subs_delivering() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;
    let mut ws = auth_and_subscribe(addr).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await
    );

    // Reload an equivalent manifest — nothing should be revoked.
    state.set_manifest(sample_manifest());

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {"x": 1}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    // First message must be the delivered event, not a revocation error.
    let ServerFrame::Event { id, payload, .. } = recv_event(&mut ws).await else {
        panic!("expected event after a no-op reload");
    };
    assert_eq!(id, "s1");
    assert_eq!(payload["x"], 1);
}
