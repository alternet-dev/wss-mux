use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};

use crate::common::{
    connect_ws, poll_until, recv_event, send_frame, sign_token, spawn_server, test_state,
    test_state_with_manifest, PUSH_TOKEN,
};

/// CBOR-encode a `{"events":[...]}` batch the way a peer (or PR4's
/// producer-side relay) will: serialize canonical envelopes, no
/// envelope-path coupling.
fn cbor_batch(value: &serde_json::Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf).expect("cbor encode");
    buf
}

#[tokio::test]
async fn relay_receive_delivers_to_local_subscribers() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    let mut client = connect_ws(addr).await;
    send_frame(
        &mut client,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;
    send_frame(
        &mut client,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
        },
    )
    .await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "subscription did not register"
    );

    let body = cbor_batch(&serde_json::json!({
        "events": [
            {"stream": "chat_messages", "key": "room-42",
             "payload": {"from": "peer", "text": "relayed"}}
        ]
    }));
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .header("Content-Type", "application/cbor")
        .body(body)
        .send()
        .await
        .expect("relay post");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    match recv_event(&mut client).await {
        ServerFrame::Event {
            stream,
            key,
            payload,
            ..
        } => {
            assert_eq!(stream, "chat_messages");
            assert_eq!(key.as_deref(), Some("room-42"));
            assert_eq!(
                payload,
                serde_json::json!({"from": "peer", "text": "relayed"})
            );
        }
        other => panic!("expected event frame, got {other:?}"),
    }
}

#[tokio::test]
async fn relay_receive_requires_push_token() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let body = cbor_batch(&serde_json::json!({"events": []}));
    let http = reqwest::Client::new();

    let missing = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .body(body.clone())
        .send()
        .await
        .expect("post");
    assert_eq!(missing.status(), reqwest::StatusCode::UNAUTHORIZED);

    let wrong = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", "Bearer not-the-token")
        .body(body)
        .send()
        .await
        .expect("post");
    assert_eq!(wrong.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn relay_receive_rejects_invalid_cbor() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .body(vec![0xff, 0x00, 0x13, 0x37])
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn relay_receive_unknown_stream_is_404_all_or_nothing() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let body = cbor_batch(&serde_json::json!({
        "events": [
            {"stream": "chat_messages", "payload": {}},
            {"stream": "no_such_stream", "payload": {}}
        ]
    }));
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .body(body)
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn relay_receive_without_manifest_returns_503() {
    let state = test_state(); // server up, manifest never set
    let addr = spawn_server(state).await;
    let body = cbor_batch(&serde_json::json!({
        "events": [{"stream": "chat_messages", "payload": {}}]
    }));
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .body(body)
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
}
