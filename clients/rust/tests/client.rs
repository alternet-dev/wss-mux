//! Behavior tests for the Rust SDK against a mock WebSocket server.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use wss_mux_client::{ErrorCode, ReconnectOptions, WssMuxClient, WssMuxError};

/// A minimal wss-mux mock server. Records every frame received on every
/// connection and exposes the connected `Sender` halves so tests can
/// push frames back to the client.
struct MockServer {
    url: String,
    state: Arc<MockState>,
    _accept_task: tokio::task::JoinHandle<()>,
}

#[derive(Default)]
struct MockState {
    frames: Mutex<Vec<Value>>,
    clients: Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>,
}

impl MockServer {
    /// Spawn a mock server on an ephemeral port and return a handle.
    /// The server accepts every upgrade attempt with the wss-mux JSON
    /// subprotocol.
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let url = format!("ws://127.0.0.1:{port}");
        let state = Arc::new(MockState::default());
        let accept_state = state.clone();

        let accept_task = tokio::spawn(async move {
            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => break,
                };
                let conn_state = accept_state.clone();
                tokio::spawn(async move {
                    let ws = match tokio_tungstenite::accept_async(stream).await {
                        Ok(w) => w,
                        Err(_) => return,
                    };
                    let (mut writer, mut reader) = ws.split();

                    // Outbound channel: tests push messages through here
                    // and a tiny pump task forwards them to the client.
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
                    conn_state.clients.lock().await.push(tx);

                    let pump = tokio::spawn(async move {
                        while let Some(m) = rx.recv().await {
                            if writer.send(m).await.is_err() {
                                break;
                            }
                        }
                    });

                    while let Some(msg) = reader.next().await {
                        match msg {
                            Ok(Message::Text(t)) => {
                                if let Ok(v) = serde_json::from_str::<Value>(&t) {
                                    conn_state.frames.lock().await.push(v);
                                }
                            }
                            Ok(Message::Close(_)) | Err(_) => break,
                            _ => {}
                        }
                    }
                    pump.abort();
                });
            }
        });

        MockServer {
            url,
            state,
            _accept_task: accept_task,
        }
    }

    async fn frames(&self) -> Vec<Value> {
        self.state.frames.lock().await.clone()
    }

    /// Send a raw message to client #0. Most tests have a single client.
    async fn send_to_client_0(&self, msg: Message) {
        let clients = self.state.clients.lock().await;
        if let Some(c) = clients.first() {
            let _ = c.send(msg);
        }
    }

    async fn client_count(&self) -> usize {
        self.state.clients.lock().await.len()
    }

    /// Close every server-side client with `code`/`reason`. Triggers the
    /// SDK's reconnect path when the client is still alive.
    async fn close_all_clients(&self, code: CloseCode, reason: &str) {
        let clients = self.state.clients.lock().await;
        let reason = reason.to_string();
        for c in clients.iter() {
            let _ = c.send(Message::Close(Some(CloseFrame {
                code,
                reason: reason.clone().into(),
            })));
        }
    }
}

fn fast_reconnect() -> ReconnectOptions {
    ReconnectOptions {
        max_attempts: None,
        initial_backoff: Duration::from_millis(10),
        max_backoff: Duration::from_millis(50),
        backoff_multiplier: 2.0,
    }
}

async fn make_client(url: &str) -> WssMuxClient {
    WssMuxClient::builder()
        .url(url)
        .get_token(|| async { Ok("test-token".to_string()) })
        .build()
        .await
        .expect("build")
}

async fn wait_frames(srv: &MockServer, n: usize) {
    for _ in 0..50 {
        if srv.frames().await.len() >= n {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!(
        "timed out waiting for {n} frames; got {}",
        srv.frames().await.len()
    );
}

// --- tests ---

#[tokio::test]
async fn connects_and_sends_auth_frame() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    wait_frames(&srv, 1).await;
    let frames = srv.frames().await;
    assert_eq!(frames[0]["type"], "auth");
    assert_eq!(frames[0]["token"], "test-token");

    client.close().await.expect("close");
}

#[tokio::test]
async fn subscribe_sends_subscribe_frame_with_key() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    let _sub = client
        .subscribe("chat", Some("room-1"))
        .await
        .expect("subscribe");
    wait_frames(&srv, 2).await;

    let frames = srv.frames().await;
    let sub = &frames[1];
    assert_eq!(sub["type"], "subscribe");
    assert_eq!(sub["stream"], "chat");
    assert_eq!(sub["key"], "room-1");
    let id = sub["id"].as_str().expect("id is string");
    assert!(id.starts_with("sub-"), "id was {id}");

    client.close().await.expect("close");
}

#[tokio::test]
async fn subscribe_without_key_omits_field() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    let _sub = client.subscribe("chat", None).await.expect("subscribe");
    wait_frames(&srv, 2).await;

    let frames = srv.frames().await;
    assert_eq!(frames[1]["type"], "subscribe");
    assert!(frames[1].get("key").is_none(), "key should be omitted");

    client.close().await.expect("close");
}

#[tokio::test]
async fn event_frame_is_delivered_to_matching_subscription() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    let mut sub = client
        .subscribe("chat", Some("room-1"))
        .await
        .expect("subscribe");
    wait_frames(&srv, 2).await;

    let id = srv.frames().await[1]["id"].as_str().unwrap().to_owned();
    srv.send_to_client_0(Message::Text(
        json!({
            "type": "event",
            "id": id,
            "stream": "chat",
            "key": "room-1",
            "payload": {"text": "hi"},
        })
        .to_string(),
    ))
    .await;

    let event = tokio::time::timeout(Duration::from_secs(1), sub.recv())
        .await
        .expect("recv timed out")
        .expect("channel closed")
        .expect("event frame");
    assert_eq!(event.stream, "chat");
    assert_eq!(event.key.as_deref(), Some("room-1"));
    assert_eq!(event.payload["text"], "hi");

    client.close().await.expect("close");
}

#[tokio::test]
async fn fatal_error_frame_terminates_subscription() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    let mut sub = client.subscribe("chat", None).await.expect("subscribe");
    wait_frames(&srv, 2).await;
    let id = srv.frames().await[1]["id"].as_str().unwrap().to_owned();

    srv.send_to_client_0(Message::Text(
        json!({
            "type": "error",
            "code": "unauthorized_subscribe",
            "message": "no audience match",
            "id": id,
        })
        .to_string(),
    ))
    .await;

    let item = tokio::time::timeout(Duration::from_secs(1), sub.recv())
        .await
        .expect("recv timed out")
        .expect("channel closed");
    let err = item.expect_err("should be error");
    match err {
        WssMuxError::Protocol { code, .. } => {
            assert_eq!(code, ErrorCode::UnauthorizedSubscribe)
        }
        other => panic!("wrong error: {other:?}"),
    }

    // Next recv should be None (channel closed).
    let after = tokio::time::timeout(Duration::from_millis(100), sub.recv()).await;
    assert!(after.is_ok() && after.unwrap().is_none());

    client.close().await.expect("close");
}

#[tokio::test]
async fn reconnects_and_replays_subscriptions() {
    let srv = MockServer::start().await;
    let client = WssMuxClient::builder()
        .url(&srv.url)
        .get_token(|| async { Ok("test-token".to_string()) })
        .reconnect(fast_reconnect())
        .build()
        .await
        .expect("build");

    let _sub = client.subscribe("chat", Some("room-1")).await.expect("sub");
    wait_frames(&srv, 2).await;
    assert_eq!(srv.client_count().await, 1);

    // Sever the connection from the server side. The SDK should
    // reconnect, re-auth, and replay the subscription.
    srv.close_all_clients(CloseCode::Abnormal, "test forced close")
        .await;

    // Auth + subscribe land on the second connection, in addition to the
    // first connection's two frames. Wait for the total to climb to 4.
    wait_frames(&srv, 4).await;

    let frames = srv.frames().await;
    assert_eq!(frames[0]["type"], "auth", "frame 0");
    assert_eq!(frames[1]["type"], "subscribe", "frame 1");
    assert_eq!(frames[2]["type"], "auth", "frame 2 — re-auth on reconnect");
    assert_eq!(frames[3]["type"], "subscribe", "frame 3 — replayed sub");
    assert_eq!(frames[3]["stream"], "chat");
    assert_eq!(frames[3]["key"], "room-1");

    client.close().await.expect("close");
}

#[tokio::test]
async fn close_code_4401_refreshes_token() {
    // We want to assert get_token() is invoked TWICE: once on initial
    // connect, and again after the server closes with 4401.
    let srv = MockServer::start().await;
    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let call_count_clone = call_count.clone();
    let client = WssMuxClient::builder()
        .url(&srv.url)
        .get_token(move || {
            let c = call_count_clone.clone();
            async move {
                let n = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(format!("token-{n}"))
            }
        })
        .reconnect(fast_reconnect())
        .build()
        .await
        .expect("build");

    wait_frames(&srv, 1).await;
    assert_eq!(srv.frames().await[0]["token"], "token-0");

    srv.close_all_clients(CloseCode::Library(4401), "expired_token")
        .await;

    // After reconnect, the SDK should have called get_token() again.
    wait_frames(&srv, 2).await;
    let frames = srv.frames().await;
    assert_eq!(frames[1]["type"], "auth");
    assert_eq!(
        frames[1]["token"], "token-1",
        "got_token was not invoked on 4401"
    );
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);

    client.close().await.expect("close");
}

#[tokio::test]
async fn dropping_subscription_sends_unsubscribe() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    let sub = client
        .subscribe("chat", Some("room-1"))
        .await
        .expect("subscribe");
    wait_frames(&srv, 2).await;

    drop(sub);
    wait_frames(&srv, 3).await;
    let frames = srv.frames().await;
    assert_eq!(frames[2]["type"], "unsubscribe");

    client.close().await.expect("close");
}
