use std::path::Path;
use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::manifest::Manifest;
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_error_frame, recv_event, send_frame, sign_token, spawn_server,
    test_state, PUSH_TOKEN,
};

const WILDCARD_MANIFEST: &str = r#"
version: 1
streams:
  - stream: chat_messages
    audience: [role:*]
  - stream: public_feed
    audience: ["*"]
"#;

fn state_with_wildcards() -> AppState {
    let state = test_state();
    state.set_manifest(
        Manifest::from_str(WILDCARD_MANIFEST, Path::new("wildcards.yaml")).expect("manifest"),
    );
    state
}

async fn subscribe(
    addr: std::net::SocketAddr,
    principals: &[&str],
    stream: &str,
) -> crate::common::Ws {
    let mut ws = connect_ws(addr).await;
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign_token(principals),
        },
    )
    .await;
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: stream.into(),
            key: None,
        },
    )
    .await;
    ws
}

#[tokio::test]
async fn trailing_wildcard_admits_matching_prefix() {
    let state = state_with_wildcards();
    let addr = spawn_server(state.clone()).await;
    let mut ws = subscribe(addr, &["role:member"], "chat_messages").await;

    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "role:member should be admitted to a role:* audience"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages", "payload": {"ok": true}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    match recv_event(&mut ws).await {
        ServerFrame::Event { stream, .. } => assert_eq!(stream, "chat_messages"),
        other => panic!("expected event, got {other:?}"),
    }
}

#[tokio::test]
async fn trailing_wildcard_denies_non_prefix_principal() {
    let state = state_with_wildcards();
    let addr = spawn_server(state).await;
    let mut ws = subscribe(addr, &["tenant:acme"], "chat_messages").await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthorized_subscribe");
    assert_eq!(id.as_deref(), Some("s1"));
}

#[tokio::test]
async fn bare_wildcard_admits_any_authenticated_connection() {
    let state = state_with_wildcards();
    let addr = spawn_server(state.clone()).await;

    // A principal that matches no specific grant, and a token with no
    // principals at all — both are admitted by the bare `*`.
    for principals in [&["tenant:zzz"][..], &[][..]] {
        let ws = subscribe(addr, principals, "public_feed").await;
        assert!(
            poll_until(Duration::from_secs(2), || {
                state.registry().binding_count("public_feed") >= 1
            })
            .await,
            "bare * must admit any authenticated connection (principals={principals:?})"
        );
        // Drop the socket before the next iteration so the binding count
        // assertion is unambiguous on a fresh connection.
        drop(ws);
        assert!(
            poll_until(Duration::from_secs(2), || {
                state.registry().binding_count("public_feed") == 0
            })
            .await,
            "binding should clear after disconnect"
        );
    }
}
