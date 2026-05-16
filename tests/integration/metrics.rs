use std::time::Duration;

use wss_mux::envelope::ClientFrame;

use crate::common::{
    connect_ws, poll_metrics_for, poll_until, recv_event, send_frame, sign_token, spawn_server,
    test_state_with_manifest, PUSH_TOKEN,
};

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
