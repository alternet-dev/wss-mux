use std::time::Duration;

use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use wss_mux::envelope::{ClientFrame, ServerFrame};

use crate::common::{
    connect_ws, connect_ws_cbor, poll_until, recv_close_code, recv_event, recv_frame_cbor,
    send_frame, send_frame_cbor, sign_token, spawn_server, test_state_with_manifest, PUSH_TOKEN,
};

#[tokio::test]
async fn cbor_round_trips_auth_subscribe_and_event() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    let mut ws = connect_ws_cbor(addr).await;
    send_frame_cbor(
        &mut ws,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;
    send_frame_cbor(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-7".into()),
        },
    )
    .await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "key": "room-7",
            "payload": {"from": "alice", "text": "hi"}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event {
        id,
        stream,
        key,
        payload,
    } = recv_frame_cbor(&mut ws).await
    else {
        panic!("expected CBOR event frame");
    };
    assert_eq!(id, "s1");
    assert_eq!(stream, "chat_messages");
    assert_eq!(key.as_deref(), Some("room-7"));
    assert_eq!(payload, serde_json::json!({"from": "alice", "text": "hi"}));
}

#[tokio::test]
async fn text_frame_on_cbor_connection_is_bad_frame_and_closes_4400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws_cbor(addr).await;
    // A JSON text frame on a CBOR-negotiated connection is a wire-type
    // mismatch → bad_frame, close 4400.
    ws.send(WsMessage::Text(r#"{"type":"auth","token":"x"}"#.into()))
        .await
        .expect("send");

    let ServerFrame::Error { code, .. } = recv_frame_cbor(&mut ws).await else {
        panic!("expected CBOR error frame");
    };
    assert_eq!(code, "bad_frame");
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn cbor_unknown_frame_type_is_distinguished_from_bad_frame() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws_cbor(addr).await;
    // Valid CBOR map with an unrecognized `type` → unknown_frame_type,
    // not bad_frame (the two-stage parse must survive the binary path).
    let mut buf = Vec::new();
    ciborium::into_writer(&serde_json::json!({"type": "nope"}), &mut buf).unwrap();
    ws.send(WsMessage::Binary(buf)).await.expect("send");

    let ServerFrame::Error { code, .. } = recv_frame_cbor(&mut ws).await else {
        panic!("expected CBOR error frame");
    };
    assert_eq!(code, "unknown_frame_type");
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn json_client_still_works_alongside_cbor() {
    // Regression: the default wss-mux (JSON) path is unchanged.
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

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
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {"json": true}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event { payload, .. } = recv_event(&mut ws).await else {
        panic!("expected JSON event frame");
    };
    assert_eq!(payload, serde_json::json!({"json": true}));
}
