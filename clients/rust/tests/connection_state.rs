//! Connection-state observability tests (parity with the TS SDK's
//! `onStateChange`). Verifies the public `state()` snapshot + the
//! `state_changes()` `watch::Receiver` track the same transitions
//! the TS SDK emits, using the same enum vocabulary.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use wss_mux_client::{ConnectionState, ReconnectOptions, WssMuxClient};

// --- mock infra (inline copy; matches tests/client.rs's pattern) ---

struct MockServer {
    url: String,
    state: Arc<MockState>,
    _accept: tokio::task::JoinHandle<()>,
}

#[derive(Default)]
struct MockState {
    clients: Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>,
    _frames: Mutex<Vec<Value>>,
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
                                    conn_state._frames.lock().await.push(v);
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
        .reconnect(fast_reconnect())
        .build()
        .await
        .expect("build")
}

// --- tests ---

#[tokio::test]
async fn build_lands_client_in_ready_state() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;
    assert_eq!(client.state(), ConnectionState::Ready);
    client.close().await.expect("close");
}

#[tokio::test]
async fn close_transitions_to_closed() {
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;
    client.clone().close().await.expect("close");
    assert_eq!(client.state(), ConnectionState::Closed);
}

#[tokio::test]
async fn state_changes_yields_a_transition_on_close() {
    // `state_changes` returns a `watch::Receiver`, which coalesces
    // rapid transitions: a consumer parked on `changed().await` only
    // ever reads the *latest* value when it wakes. We can't assert
    // intermediate states like `Closing` are individually observed —
    // they're emitted but may be overwritten before the receiver
    // polls. We *can* assert:
    //   - the initial borrow matches `state()`,
    //   - `changed().await` returns at least once after a close,
    //   - the borrowed value after the change is `Closed`.
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;

    let mut rx = client.state_changes();
    assert_eq!(*rx.borrow(), ConnectionState::Ready);
    assert_eq!(client.state(), ConnectionState::Ready);

    let close_handle = tokio::spawn({
        let c = client.clone();
        async move { c.close().await }
    });

    // Walk transitions until we land on `Closed`.
    let outcome = tokio::time::timeout(Duration::from_secs(2), async {
        while rx.changed().await.is_ok() {
            if *rx.borrow() == ConnectionState::Closed {
                return;
            }
        }
    })
    .await;
    assert!(outcome.is_ok(), "did not observe Closed via state_changes");
    close_handle.await.expect("close join").expect("close ok");
    assert_eq!(client.state(), ConnectionState::Closed);
}

#[tokio::test]
async fn server_close_drives_reconnecting_then_back_to_ready() {
    // The TS SDK's onStateChange goes Ready → Reconnecting → Connecting
    // → Authenticating → Ready across a transient server-side close.
    // The Rust SDK should walk the same sequence.
    let srv = MockServer::start().await;
    let client = make_client(&srv.url).await;
    assert_eq!(client.state(), ConnectionState::Ready);

    let mut rx = client.state_changes();
    srv.close_all_clients(CloseCode::Abnormal, "test forced close")
        .await;

    // Walk transitions until Ready lands again (or timeout).
    let mut saw_reconnecting = false;
    let outcome = tokio::time::timeout(Duration::from_secs(2), async {
        while rx.changed().await.is_ok() {
            let s = *rx.borrow();
            if s == ConnectionState::Reconnecting {
                saw_reconnecting = true;
            }
            if s == ConnectionState::Ready && saw_reconnecting {
                return;
            }
        }
    })
    .await;

    assert!(outcome.is_ok(), "did not land in Ready after Reconnecting");
    assert!(saw_reconnecting, "did not pass through Reconnecting");
    assert_eq!(client.state(), ConnectionState::Ready);

    client.close().await.expect("close");
}
