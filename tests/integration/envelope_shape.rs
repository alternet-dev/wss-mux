use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_event, sample_manifest, send_frame, sign_token, spawn_server,
    test_config, PUSH_TOKEN,
};

#[tokio::test]
async fn custom_paths_extract_and_deliver() {
    let mut cfg = test_config();
    cfg.envelope_stream_path = "meta.topic".into();
    cfg.envelope_key_path = "meta.room".into();
    cfg.envelope_payload_path = "data".into();
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
            key: Some("42".into()),
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
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "meta": {"topic": "chat_messages", "room": "42"},
            "data": {"text": "hi"}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event {
        stream,
        key,
        payload,
        ..
    } = recv_event(&mut ws).await
    else {
        panic!("expected event");
    };
    assert_eq!(stream, "chat_messages");
    assert_eq!(key.as_deref(), Some("42"));
    assert_eq!(payload, serde_json::json!({"text": "hi"}));
}

#[tokio::test]
async fn default_paths_classic_body_still_works() {
    // Regression: with default config the classic {stream,key,payload}
    // body must keep working byte-for-byte.
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
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {"classic": true}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event { payload, .. } = recv_event(&mut ws).await else {
        panic!("expected event");
    };
    assert_eq!(payload, serde_json::json!({"classic": true}));
}

#[tokio::test]
async fn body_not_matching_custom_paths_is_400() {
    let mut cfg = test_config();
    cfg.envelope_stream_path = "meta.topic".into();
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    // Classic body has no meta.topic → stream extraction fails → 400.
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn batch_with_custom_paths_delivers_all() {
    let mut cfg = test_config();
    cfg.envelope_stream_path = "t".into();
    cfg.envelope_payload_path = "d".into();
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
        .await
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "events": [
                {"t": "chat_messages", "d": {"i": 1}},
                {"t": "chat_messages", "d": {"i": 2}}
            ]
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let mut got = Vec::new();
    for _ in 0..2 {
        let ServerFrame::Event { payload, .. } = recv_event(&mut ws).await else {
            panic!("expected event");
        };
        got.push(payload["i"].as_u64().unwrap());
    }
    got.sort();
    assert_eq!(got, vec![1, 2]);
}
