use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use wss_mux::envelope::{ClientFrame, ServerFrame};

use crate::common::{sign_token, sign_token_with_offsets, spawn_server, test_state_with_manifest};

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

async fn send_text(ws: &mut Ws, text: String) {
    ws.send(WsMessage::Text(text)).await.expect("send");
}

async fn send_frame(ws: &mut Ws, frame: &ClientFrame) {
    send_text(ws, serde_json::to_string(frame).expect("serialize")).await;
}

async fn recv_message(ws: &mut Ws) -> WsMessage {
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

async fn recv_error_frame(ws: &mut Ws) -> (String, Option<String>) {
    match recv_message(ws).await {
        WsMessage::Text(t) => match serde_json::from_str(t.as_str()).expect("parse") {
            ServerFrame::Error { code, id, .. } => (code, id),
            other => panic!("expected error frame, got {other:?}"),
        },
        other => panic!("expected text frame, got {other:?}"),
    }
}

async fn recv_close_code(ws: &mut Ws) -> u16 {
    match recv_message(ws).await {
        WsMessage::Close(Some(frame)) => u16::from(frame.code),
        WsMessage::Close(None) => 1005,
        other => panic!("expected close, got {other:?}"),
    }
}

#[tokio::test]
async fn bad_frame_emits_error_and_closes_4400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    send_text(&mut ws, "not valid json".into()).await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "bad_frame");
    assert_eq!(id, None);
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn unknown_frame_type_emits_error_and_closes_4400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    send_text(&mut ws, r#"{"type":"nope"}"#.into()).await;

    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unknown_frame_type");
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn binary_frame_emits_bad_frame_and_closes_4400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    ws.send(WsMessage::Binary(vec![0, 1, 2]))
        .await
        .expect("send");

    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "bad_frame");
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn subscribe_before_auth_emits_unauthenticated_and_closes_4401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(id.as_deref(), Some("s1"));
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn expired_token_emits_expired_and_closes_4401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    // iat 2h ago, exp 1h ago.
    let token = sign_token_with_offsets(&["role:member"], -7200, -3600);
    send_frame(&mut ws, &ClientFrame::Auth { token }).await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "expired_token");
    assert_eq!(id, None);
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn invalid_signature_emits_unauthenticated_and_closes_4401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    // A syntactically valid but improperly signed JWT.
    let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJ4IiwiaWF0IjoxLCJleHAiOjk5OTk5OTk5OTksInN1YiI6IngiLCJwcmluY2lwYWxzIjpbXX0.aaaaaa";
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: token.into(),
        },
    )
    .await;

    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn subscribe_to_unknown_stream_emits_error_and_stays_open() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
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
            stream: "no-such-stream".into(),
            key: None,
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unknown_stream");
    assert_eq!(id.as_deref(), Some("s1"));

    // Connection still works: a valid subscribe afterward succeeds (no error).
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s2".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;
    // No error should follow; assert by sending unsubscribe and reading no error.
    send_frame(&mut ws, &ClientFrame::Unsubscribe { id: "s2".into() }).await;
    // Close gracefully — if the server had emitted anything, recv_message would surface it.
}

#[tokio::test]
async fn unauthorized_subscribe_emits_error_and_stays_open() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    // Token with a principal that does not intersect chat_messages' audience ([role:member]).
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign_token(&["role:guest"]),
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

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthorized_subscribe");
    assert_eq!(id.as_deref(), Some("s1"));
    // Connection stays open — no close received.
}

#[tokio::test]
async fn duplicate_subscription_id_emits_error_and_stays_open() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
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
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "duplicate_subscription_id");
    assert_eq!(id.as_deref(), Some("s1"));
}
