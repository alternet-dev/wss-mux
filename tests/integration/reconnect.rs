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

async fn poll_until<F: FnMut() -> bool>(timeout: Duration, mut cond: F) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if cond() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    cond()
}

async fn subscribe_chat_messages(ws: &mut Ws) {
    send_frame(
        ws,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;
    send_frame(
        ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;
}

#[tokio::test]
async fn dropping_ws_releases_subscriptions() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    {
        let mut ws = connect_ws(addr).await;
        subscribe_chat_messages(&mut ws).await;

        assert!(
            poll_until(Duration::from_secs(2), || {
                state.registry().binding_count("chat_messages") >= 1
            })
            .await,
            "subscription did not register"
        );
        // ws drops at end of block, closing the connection.
    }

    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") == 0
        })
        .await,
        "registry was not cleaned up after disconnect"
    );
}

#[tokio::test]
async fn reconnect_resubscribes_and_receives_new_events() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    {
        let mut ws = connect_ws(addr).await;
        subscribe_chat_messages(&mut ws).await;
        assert!(
            poll_until(Duration::from_secs(2), || {
                state.registry().binding_count("chat_messages") >= 1
            })
            .await,
            "first subscription did not register"
        );
    }

    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") == 0
        })
        .await,
        "registry not cleaned up before reconnect"
    );

    // Reconnect on a fresh socket.
    let mut ws = connect_ws(addr).await;
    subscribe_chat_messages(&mut ws).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "second subscription did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {"text": "after-reconnect"}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event { id, payload, .. } = recv_event(&mut ws).await else {
        panic!("expected event after reconnect");
    };
    assert_eq!(id, "s1");
    assert_eq!(payload["text"], "after-reconnect");
}
