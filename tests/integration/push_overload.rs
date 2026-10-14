//! Opt-in HTTP producer overload signaling.

use std::time::Duration;

use futures_util::StreamExt;
use wss_mux::envelope::ClientFrame;
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_event, send_frame, sign_token, spawn_server, test_config,
    test_state_with_manifest, PUSH_TOKEN,
};

fn overloaded_state() -> AppState {
    let mut cfg = test_config();
    cfg.push_overload_rate_per_sec = 1;
    cfg.push_overload_burst = 1;
    let state = AppState::new(cfg);
    state.set_manifest(crate::common::sample_manifest());
    state
}

fn event() -> serde_json::Value {
    serde_json::json!({"stream": "chat_messages", "payload": {"ok": true}})
}

#[tokio::test]
async fn overload_signal_is_opt_in_and_leaves_default_pushes_unlimited() {
    let state = test_state_with_manifest();
    assert!(state.push_overload_limiter().is_none());
    let addr = spawn_server(state).await;
    let http = reqwest::Client::new();

    for _ in 0..3 {
        let response = http
            .post(format!("http://{addr}/events"))
            .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
            .json(&event())
            .send()
            .await
            .expect("push");
        assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
    }
}

#[tokio::test]
async fn exhausted_push_returns_retryable_overload_without_dispatching() {
    let state = overloaded_state();
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
            id: "sub".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") == 1
        })
        .await
    );

    let http = reqwest::Client::new();
    let accepted = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&event())
        .send()
        .await
        .expect("accepted push");
    assert_eq!(accepted.status(), reqwest::StatusCode::NO_CONTENT);
    let _ = recv_event(&mut ws).await;

    let rejected = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&event())
        .send()
        .await
        .expect("rejected push response");
    assert_eq!(rejected.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        rejected
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .unwrap(),
        "1"
    );
    assert_eq!(
        rejected.headers().get(reqwest::header::CONNECTION).unwrap(),
        "close"
    );
    assert_eq!(state.metrics().push_overload_rejected.get(), 1);

    assert!(
        tokio::time::timeout(Duration::from_millis(200), ws.next())
            .await
            .is_err(),
        "a rejected request must not reach subscribers"
    );
}

#[tokio::test]
async fn batch_pushes_share_the_overload_budget_after_authentication() {
    let state = overloaded_state();
    let addr = spawn_server(state.clone()).await;
    let http = reqwest::Client::new();

    let unauthorized = http
        .post(format!("http://{addr}/events"))
        .json(&event())
        .send()
        .await
        .expect("unauthorized push");
    assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);

    let accepted = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&event())
        .send()
        .await
        .expect("accepted push");
    assert_eq!(accepted.status(), reqwest::StatusCode::NO_CONTENT);

    let rejected = http
        .post(format!("http://{addr}/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({"events": [event()]}))
        .send()
        .await
        .expect("rejected batch");
    assert_eq!(rejected.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(state.metrics().push_overload_rejected.get(), 1);
}
