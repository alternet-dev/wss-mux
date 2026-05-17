use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_event, sample_manifest, send_frame, sign_token, spawn_server,
    test_config, PUSH_TOKEN,
};

/// v0.4 per-subscription model: a subscriber that keeps up receives far
/// more than `queue_depth` events on a single subscription.
///
/// This only holds if the writer releases each subscription's in-flight
/// slot after writing the frame. Without release, the dispatcher's
/// per-subscription reservation count grows monotonically and the
/// subscription "overflows" after just `queue_depth` cumulative events —
/// even though the client is draining every one.
#[tokio::test]
async fn steady_consumer_receives_far_more_than_queue_depth() {
    let mut cfg = test_config();
    cfg.queue_depth = 4; // small cap so a missing release trips fast
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
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

    let http = reqwest::Client::new();
    let total = 20; // 5x the cap
    for i in 0..total {
        let resp = http
            .post(format!("http://{addr}/v1/events"))
            .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
            .json(&serde_json::json!({
                "stream": "chat_messages",
                "payload": {"i": i}
            }))
            .send()
            .await
            .expect("push");
        assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

        let frame = recv_event(&mut ws).await;
        let ServerFrame::Event { id, payload, .. } = frame else {
            panic!(
                "event {i}: expected an Event frame, got {frame:?} — \
                 a steady consumer must never overflow"
            );
        };
        assert_eq!(id, "s1");
        assert_eq!(payload["i"], i);
    }

    // Subscription still alive, connection still open.
    assert_eq!(state.registry().binding_count("chat_messages"), 1);
}

/// Global `WSS_MUX_QUEUE_DEPTH=0` ⇒ the per-connection channel is
/// unbounded: a burst far larger than any bounded default is buffered
/// and fully delivered with no overflow, even though the client does
/// not read until the whole burst has been pushed.
///
/// Deterministic: an unbounded channel never drops, so every pushed
/// event is still there when the client finally drains.
#[tokio::test]
async fn global_queue_depth_zero_is_unbounded_no_drops() {
    let mut cfg = test_config();
    cfg.queue_depth = 0; // explicit "unlimited"
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
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

    // Push a big burst WITHOUT reading any of it yet.
    let http = reqwest::Client::new();
    let total = 200;
    for i in 0..total {
        let resp = http
            .post(format!("http://{addr}/v1/events"))
            .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
            .json(&serde_json::json!({
                "stream": "chat_messages",
                "payload": {"i": i}
            }))
            .send()
            .await
            .expect("push");
        assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);
    }

    // Now drain: every event is present, in order, none dropped.
    for i in 0..total {
        let frame = recv_event(&mut ws).await;
        let ServerFrame::Event { id, payload, .. } = frame else {
            panic!(
                "event {i}: expected an Event frame, got {frame:?} — \
                 an unbounded connection must never overflow"
            );
        };
        assert_eq!(id, "s1");
        assert_eq!(payload["i"], i);
    }
    assert_eq!(state.registry().binding_count("chat_messages"), 1);
}
