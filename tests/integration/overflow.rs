use std::net::SocketAddr;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use wss_mux::envelope::{ClientFrame, ServerFrame};

use crate::common::{sign_token, spawn_server, test_state_with_manifest};

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

/// End-to-end test of the user-visible overflow behavior: when overflow is
/// triggered for a connection, the client receives an `error{code:"overflow"}`
/// text frame followed by a WebSocket close with code 4429, and the registry
/// is cleaned up afterward.
///
/// We invoke `state.trigger_overflow` directly rather than racing the
/// dispatcher's try_send against the OS TCP send buffer — that race depends
/// on the platform's default SO_SNDBUF and is unreliable on Linux CI. The
/// dispatcher's Full -> trigger_overflow wiring is covered by a unit test in
/// `src/dispatcher.rs`.
#[tokio::test]
async fn overflow_emits_error_frame_and_closes_4429() {
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
        .await,
        "subscription did not register"
    );

    // First (and only) connection has conn_id 1; the AtomicU64 in AppState
    // starts at 1.
    state.trigger_overflow(1);

    let mut saw_overflow = false;
    let mut close_code: Option<u16> = None;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && close_code.is_none() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match tokio::time::timeout(remaining, ws.next()).await {
            Ok(Some(Ok(WsMessage::Text(t)))) => {
                if let Ok(ServerFrame::Error { code, .. }) =
                    serde_json::from_str::<ServerFrame>(t.as_str())
                {
                    if code == "overflow" {
                        saw_overflow = true;
                    }
                }
            }
            Ok(Some(Ok(WsMessage::Close(Some(f))))) => close_code = Some(u16::from(f.code)),
            Ok(Some(Ok(WsMessage::Close(None)))) => close_code = Some(1005),
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(_))) | Ok(None) => break,
            Err(_) => break,
        }
    }

    assert!(saw_overflow, "expected overflow error frame");
    assert_eq!(close_code, Some(4429), "expected close code 4429");

    assert!(
        poll_until(Duration::from_secs(5), || {
            state.registry().binding_count("chat_messages") == 0
        })
        .await,
        "registry was not cleaned up after overflow"
    );
}
