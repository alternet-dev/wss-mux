use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_error_frame, recv_event, sample_manifest, send_frame, sign_token,
    spawn_server, test_config, PUSH_TOKEN,
};

#[tokio::test]
async fn flooding_is_rate_limited_then_recovers() {
    // burst 2, 1 token/sec. auth spends one token; the first flood frame
    // spends the second; everything after is throttled until a refill.
    let mut cfg = test_config();
    cfg.inbound_rate_per_sec = 1;
    cfg.inbound_burst = 2;
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

    // auth spent one of the two burst tokens. The first unsubscribe
    // (idempotent no-op, emits nothing) spends the last token; the second
    // is throttled and produces exactly one `rate_limited` error frame —
    // keeping the count exact so the later recv_event isn't shadowed by a
    // backlog of error frames.
    send_frame(&mut ws, &ClientFrame::Unsubscribe { id: "drain".into() }).await;
    send_frame(
        &mut ws,
        &ClientFrame::Unsubscribe {
            id: "throttled".into(),
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "rate_limited");
    assert_eq!(id.as_deref(), Some("throttled"));

    // Connection survives the throttling. After a refill window a fresh
    // subscribe is admitted and starts delivering.
    tokio::time::sleep(Duration::from_millis(1300)).await;
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "recovered".into(),
            stream: "chat_messages".into(),
            key: Some("rk".into()),
        },
    )
    .await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "post-refill subscribe should have been admitted"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "key": "rk",
            "payload": {"ok": true}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event { id, payload, .. } = recv_event(&mut ws).await else {
        panic!("expected event after recovery");
    };
    assert_eq!(id, "recovered");
    assert_eq!(payload, serde_json::json!({"ok": true}));
}

#[tokio::test]
async fn rate_limit_disabled_by_default_in_tests() {
    // test_config() sets rate 0 → unlimited. A rapid burst of subscribes
    // must all be admitted (no rate_limited frame).
    let state = AppState::new(test_config());
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
    for i in 0..50 {
        send_frame(
            &mut ws,
            &ClientFrame::Subscribe {
                id: format!("s{i}"),
                stream: "chat_messages".into(),
                key: None,
            },
        )
        .await;
    }
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 50
        })
        .await,
        "all 50 subscribes should be admitted with rate limiting off"
    );
}
