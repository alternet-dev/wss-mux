use std::time::Duration;

use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use wss_mux::envelope::{ClientFrame, ServerFrame};

use crate::common::{
    connect_ws, poll_until, recv_event, send_frame, sign_token, spawn_server,
    test_state_with_manifest, PUSH_TOKEN,
};

#[tokio::test]
async fn push_event_fans_out_to_subscribed_clients() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    let mut client_a = connect_ws(addr).await;
    let mut client_b = connect_ws(addr).await;

    for client in [&mut client_a, &mut client_b] {
        send_frame(
            client,
            &ClientFrame::Auth {
                token: sign_token(&["role:member"]),
            },
        )
        .await;
        send_frame(
            client,
            &ClientFrame::Subscribe {
                id: "s1".into(),
                stream: "chat_messages".into(),
                key: Some("room-42".into()),
            },
        )
        .await;
    }

    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 2
        })
        .await,
        "subscriptions did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "key": "room-42",
            "payload": {"from": "alice", "text": "hi"}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    for client in [&mut client_a, &mut client_b] {
        let event = recv_event(client).await;
        let ServerFrame::Event {
            id,
            stream,
            key,
            payload,
        } = event
        else {
            panic!("expected event frame, got {event:?}");
        };
        assert_eq!(id, "s1");
        assert_eq!(stream, "chat_messages");
        assert_eq!(key.as_deref(), Some("room-42"));
        assert_eq!(payload["text"], "hi");
    }
}

#[tokio::test]
async fn unknown_stream_push_returns_404() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "nope",
            "payload": {}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn push_without_bearer_token_is_401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn push_with_wrong_bearer_token_is_401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .header("Authorization", "Bearer wrong-token")
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn batch_push_delivers_all_events() {
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
        .post(format!("http://{addr}/v1/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "events": [
                {"stream": "chat_messages", "payload": {"i": 1}},
                {"stream": "chat_messages", "payload": {"i": 2}},
                {"stream": "chat_messages", "payload": {"i": 3}},
            ]
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let mut payloads = Vec::new();
    for _ in 0..3 {
        let ServerFrame::Event { payload, .. } = recv_event(&mut ws).await else {
            panic!("expected event");
        };
        payloads.push(payload["i"].as_u64().unwrap());
    }
    payloads.sort();
    assert_eq!(payloads, vec![1, 2, 3]);
}

#[tokio::test]
async fn batch_push_without_bearer_token_is_401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events/batch"))
        .json(&serde_json::json!({
            "events": [{"stream": "chat_messages", "payload": {}}]
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn batch_push_with_unknown_stream_in_any_event_is_404() {
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
        .post(format!("http://{addr}/v1/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "events": [
                {"stream": "chat_messages", "payload": {"valid": true}},
                {"stream": "not-in-manifest", "payload": {"bad": true}},
            ]
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    let timeout_res = tokio::time::timeout(Duration::from_millis(200), ws.next()).await;
    assert!(
        timeout_res.is_err(),
        "no event should be delivered when batch validation fails"
    );
}

#[tokio::test]
async fn batch_push_with_malformed_envelope_is_400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "events": [
                {"stream": "chat_messages", "payload": {}},
                {"payload": {"missing_stream": true}}
            ]
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn batch_push_with_empty_events_array_is_204() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({"events": []}))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn ws_without_subprotocol_is_rejected() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let url = format!("ws://{addr}/v1/stream");
    let req = url.into_client_request().expect("request");
    let res = tokio_tungstenite::connect_async(req).await;
    assert!(res.is_err(), "connect without subprotocol should fail");
}
