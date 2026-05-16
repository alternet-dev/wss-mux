use std::net::SocketAddr;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use wss_mux::envelope::{ClientFrame, ServerFrame};

use crate::common::{sign_token, spawn_server, test_state_with_manifest, PUSH_TOKEN};

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn connect_ws(addr: SocketAddr) -> Ws {
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

async fn send_frame(ws: &mut Ws, frame: &ClientFrame) {
    let json = serde_json::to_string(frame).expect("serialize");
    ws.send(WsMessage::Text(json)).await.expect("send");
}

async fn recv_event(ws: &mut Ws) -> ServerFrame {
    let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("recv timed out")
        .expect("stream ended")
        .expect("recv error");
    match msg {
        WsMessage::Text(t) => serde_json::from_str(t.as_str()).expect("parse frame"),
        other => panic!("expected text frame, got {other:?}"),
    }
}

#[tokio::test]
async fn push_event_fans_out_to_subscribed_clients() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    let mut client_a = connect_ws(addr).await;
    let mut client_b = connect_ws(addr).await;

    send_frame(
        &mut client_a,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;
    send_frame(
        &mut client_b,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;

    send_frame(
        &mut client_a,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
        },
    )
    .await;
    send_frame(
        &mut client_b,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
        },
    )
    .await;

    let deadline = Instant::now() + Duration::from_secs(2);
    while state.registry().binding_count("chat_messages") < 2 {
        if Instant::now() > deadline {
            panic!("subscriptions did not register within 2s");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

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

    let event_a = recv_event(&mut client_a).await;
    let event_b = recv_event(&mut client_b).await;

    for event in [&event_a, &event_b] {
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

    let deadline = Instant::now() + Duration::from_secs(2);
    while state.registry().binding_count("chat_messages") < 1 {
        if Instant::now() > deadline {
            panic!("subscription did not register");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

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

    // Subscribe so we'd see deliveries if any leaked through.
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

    let deadline = Instant::now() + Duration::from_secs(2);
    while state.registry().binding_count("chat_messages") < 1 {
        if Instant::now() > deadline {
            panic!("subscription did not register");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

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

    // No event should have been dispatched (all-or-nothing). Give the
    // server a moment, then verify nothing arrived.
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
    // Missing required `stream` field in one event.
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
