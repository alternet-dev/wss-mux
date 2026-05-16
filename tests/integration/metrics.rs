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

#[tokio::test]
async fn metrics_endpoint_returns_openmetrics_text() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    let resp = reqwest::get(format!("http://{addr}/metrics"))
        .await
        .expect("scrape");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("application/openmetrics-text"),
        "content-type was {content_type}"
    );
    let body = resp.text().await.expect("body");
    // Smoke-check: presence of each metric we registered.
    for name in [
        "wss_mux_connections",
        "wss_mux_connections_active",
        "wss_mux_subscriptions_active",
        "wss_mux_events_dispatched",
        "wss_mux_events_dropped",
        "wss_mux_send_queue_depth",
    ] {
        assert!(
            body.contains(name),
            "expected metric {name} in body, got:\n{body}"
        );
    }
}

#[tokio::test]
async fn dispatched_counter_increments_on_delivery() {
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
            "payload": {"x": 1}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let _event = recv_event(&mut ws).await;

    let body = reqwest::get(format!("http://{addr}/metrics"))
        .await
        .expect("scrape")
        .text()
        .await
        .expect("body");

    assert!(
        body.contains(r#"wss_mux_events_dispatched_total{stream="chat_messages"} 1"#),
        "expected dispatched counter to be 1 for chat_messages, got:\n{body}"
    );
    assert!(
        body.contains("wss_mux_connections_active 1"),
        "expected active connections gauge to be 1, got:\n{body}"
    );
    assert!(
        body.contains("wss_mux_subscriptions_active 1"),
        "expected active subscriptions gauge to be 1, got:\n{body}"
    );
}

async fn poll_metrics_for(addr: SocketAddr, substring: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let http = reqwest::Client::new();
    while Instant::now() < deadline {
        if let Ok(resp) = http.get(format!("http://{addr}/metrics")).send().await {
            if let Ok(body) = resp.text().await {
                if body.contains(substring) {
                    return true;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    false
}

#[tokio::test]
async fn connections_active_decrements_on_disconnect() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;

    {
        let _ws = connect_ws(addr).await;
        assert!(
            poll_metrics_for(addr, "wss_mux_connections_active 1", Duration::from_secs(2)).await,
            "active connections did not reach 1"
        );
    }

    assert!(
        poll_metrics_for(addr, "wss_mux_connections_active 0", Duration::from_secs(2)).await,
        "active connections did not return to 0 after disconnect"
    );
}
