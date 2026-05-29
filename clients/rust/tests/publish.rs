//! Behavior tests for `WssMuxClient::publish` against a mock server.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use wss_mux_client::{ErrorCode, WssMuxClient, WssMuxError};

// --- mock infra (kept lean: one source of truth lives in tests/client.rs
// for the subscribe suite; this file uses an inline copy so the suites
// can be run independently and stay easy to read) ---

struct MockServer {
    url: String,
    state: Arc<MockState>,
    _accept: tokio::task::JoinHandle<()>,
}

#[derive(Default)]
struct MockState {
    frames: Mutex<Vec<Value>>,
    clients: Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>,
    /// If Some, the server auto-rejects every publish whose stream
    /// matches the configured name with an `unauthorized_publish`
    /// error frame echoing the publish's id.
    deny_publish_stream: Mutex<Option<String>>,
}

impl MockServer {
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{port}");
        let state = Arc::new(MockState::default());
        let accept_state = state.clone();
        let accept = tokio::spawn(async move {
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
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
                    conn_state.clients.lock().await.push(tx.clone());
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
                                let v: Value = match serde_json::from_str(&t) {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                };
                                conn_state.frames.lock().await.push(v.clone());
                                // Auto-reject publishes whose stream matches.
                                if v["type"] == "publish" {
                                    let deny = conn_state.deny_publish_stream.lock().await.clone();
                                    if let Some(s) = deny {
                                        if v["stream"] == s {
                                            let err = json!({
                                                "type": "error",
                                                "code": "unauthorized_publish",
                                                "message": "denied",
                                                "id": v["id"],
                                            });
                                            let _ = tx.send(Message::Text(err.to_string()));
                                        }
                                    }
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
            _accept: accept,
        }
    }

    async fn frames(&self) -> Vec<Value> {
        self.state.frames.lock().await.clone()
    }

    async fn deny_publish_to(&self, stream: &str) {
        *self.state.deny_publish_stream.lock().await = Some(stream.to_string());
    }

    async fn send_to_client_0(&self, msg: Message) {
        let clients = self.state.clients.lock().await;
        if let Some(c) = clients.first() {
            let _ = c.send(msg);
        }
    }
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

async fn make_client(url: &str) -> WssMuxClient {
    WssMuxClient::builder()
        .url(url)
        .get_token(|| async { Ok("test-token".to_string()) })
        // 250 ms is the same default the library ships with. A tighter
        // value (e.g. 25 ms) wins under local loopback but races the
        // pump-task wakeup on GH-hosted runners — the server's error
        // frame can arrive 30–60 ms after the publish lands. The cost
        // for tests that *do* settle (rather than reject) is one
        // settle window each; the suite still runs in well under a
        // second.
        .publish_settle(Duration::from_millis(250))
        .build()
        .await
        .expect("build")
}

// --- tests ---

#[tokio::test]
async fn publish_sends_a_publish_frame_and_resolves() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    client
        .publish("chat", Some("room-42"), json!({"text": "hi"}))
        .await
        .expect("publish");

    wait_frames(&srv, 2).await;
    let frames = srv.frames().await;
    assert_eq!(frames[1]["type"], "publish");
    assert!(frames[1]["id"].as_str().unwrap().starts_with("pub-"));
    assert_eq!(frames[1]["stream"], "chat");
    assert_eq!(frames[1]["key"], "room-42");
    assert_eq!(frames[1]["payload"]["text"], "hi");

    client.close().await.expect("close");
}

#[tokio::test]
async fn publish_without_key_omits_field() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    client
        .publish("notifications", None, json!({"online": true}))
        .await
        .expect("publish");
    wait_frames(&srv, 2).await;

    let frames = srv.frames().await;
    let pub_frame = &frames[1];
    assert_eq!(pub_frame["stream"], "notifications");
    assert!(pub_frame.get("key").is_none(), "key should be omitted");

    client.close().await.expect("close");
}

#[tokio::test]
async fn publish_rejects_when_server_sends_matching_error_frame() {
    let srv = MockServer::start().await;
    srv.deny_publish_to("denied").await;
    let client = make_client(&srv.url).await;

    let err = client
        .publish("denied", None, json!({"x": 1}))
        .await
        .expect_err("publish should reject");
    match err {
        WssMuxError::Protocol { code, id, .. } => {
            assert_eq!(code, ErrorCode::UnauthorizedPublish);
            assert!(id.unwrap().starts_with("pub-"));
        }
        other => panic!("wrong error variant: {other:?}"),
    }

    client.close().await.expect("close");
}

#[tokio::test]
async fn multiple_concurrent_publishes_settle_independently() {
    let srv = MockServer::start().await;
    srv.deny_publish_to("denied").await;
    let client = make_client(&srv.url).await;

    let (a, b, c) = tokio::join!(
        client.publish("chat", None, json!({"n": 1})),
        client.publish("denied", None, json!({"n": 2})),
        client.publish("chat", None, json!({"n": 3})),
    );

    assert!(a.is_ok(), "publish #1 should resolve, got {a:?}");
    let b_err = b.expect_err("publish #2 should reject");
    assert_eq!(b_err.code(), Some(ErrorCode::UnauthorizedPublish));
    assert!(c.is_ok(), "publish #3 should resolve, got {c:?}");

    client.close().await.expect("close");
}

#[tokio::test]
async fn publish_after_close_returns_closed_error() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;
    client.clone().close().await.expect("close");

    let err = client
        .publish("chat", None, json!({"x": 1}))
        .await
        .expect_err("publish after close should fail");
    assert!(matches!(err, WssMuxError::Closed), "got {err:?}");
}

#[tokio::test]
async fn error_frame_for_unknown_publish_id_is_a_no_op() {
    // After a publish has settled, a late-arriving error frame keyed
    // to that publish id finds no pending entry and is dropped on the
    // floor (the Rust SDK has no global onError hook). The connection
    // stays up and subsequent publishes still work.
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;
    client
        .publish("chat", None, json!({"x": 1}))
        .await
        .expect("first publish");

    // Send an error frame with an unknown publish id.
    srv.send_to_client_0(Message::Text(
        json!({
            "type": "error",
            "code": "rate_limited",
            "message": "late",
            "id": "pub-9999",
        })
        .to_string(),
    ))
    .await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Connection is still alive — a follow-up publish should resolve.
    client
        .publish("chat", None, json!({"x": 2}))
        .await
        .expect("second publish");

    client.close().await.expect("close");
}

#[tokio::test]
async fn publish_rejects_when_connection_drops_mid_flight() {
    use wss_mux_client::ReconnectOptions;
    // Connection drop: pending publish should reject with a closed-
    // connection error rather than hanging forever on settle.
    let srv = MockServer::start().await;
    let client = WssMuxClient::builder()
        .url(&srv.url)
        .get_token(|| async { Ok("test-token".to_string()) })
        // Use a long settle so the drop-rejection path is the only
        // thing that can resolve this test in <1s.
        .publish_settle(Duration::from_secs(60))
        .reconnect(ReconnectOptions {
            // Cap retries so the test doesn't hang on background backoff.
            max_attempts: Some(0),
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(50),
            backoff_multiplier: 2.0,
        })
        .build()
        .await
        .expect("build");

    // Race: kick off publish, then sever the connection.
    let publish_fut = client.publish("chat", None, json!({"x": 1}));
    let server = srv.clone_for_drop();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        server.terminate_client_0().await;
    });

    let err = tokio::time::timeout(Duration::from_secs(2), publish_fut)
        .await
        .expect("publish promise hung past timeout")
        .expect_err("publish should reject");
    match err {
        WssMuxError::ConnectionClosed { .. } | WssMuxError::ReconnectExhausted => {}
        other => panic!("expected closed/exhausted, got {other:?}"),
    }
}

impl MockServer {
    /// Cheap clone for fixtures that need to schedule server-side work
    /// from a spawned task. Returns a struct that shares the same
    /// inner state but doesn't own the accept task.
    fn clone_for_drop(&self) -> MockServerHandle {
        MockServerHandle {
            state: self.state.clone(),
        }
    }
}

struct MockServerHandle {
    state: Arc<MockState>,
}

impl MockServerHandle {
    /// Send a Close frame to the client, mimicking an abrupt server
    /// shutdown. Returns once the message has been queued on the
    /// connection's outbound channel.
    async fn terminate_client_0(&self) {
        use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
        use tokio_tungstenite::tungstenite::protocol::CloseFrame;
        let clients = self.state.clients.lock().await;
        if let Some(c) = clients.first() {
            let _ = c.send(Message::Close(Some(CloseFrame {
                code: CloseCode::Abnormal,
                reason: "test forced close".into(),
            })));
        }
    }
}
