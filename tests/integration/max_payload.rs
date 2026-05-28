use std::path::Path;
use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::manifest::Manifest;
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_metrics_for, poll_until, recv_event, send_frame, sign_token, spawn_server,
    test_state, PUSH_TOKEN,
};

// `capped` admits anyone (bare `*`) and caps payloads at 64 JSON bytes.
const CAPPED_MANIFEST: &str = r#"
version: 1
streams:
  - stream: capped
    subscribe: ["*"]
    max_payload_bytes: 64
"#;

fn capped_state() -> AppState {
    let state = test_state();
    state.set_manifest(
        Manifest::from_str(CAPPED_MANIFEST, Path::new("capped.yaml")).expect("manifest"),
    );
    state
}

fn cbor_batch(value: &serde_json::Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf).expect("cbor encode");
    buf
}

// A payload whose JSON serialization comfortably exceeds 64 bytes.
fn oversized() -> serde_json::Value {
    serde_json::json!({ "blob": "x".repeat(256) })
}

#[tokio::test]
async fn push_over_cap_is_413_and_metered() {
    let state = capped_state();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({"stream": "capped", "payload": oversized()}))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);

    assert!(
        poll_metrics_for(
            addr,
            "wss_mux_events_rejected_total{reason=\"payload_too_large\"}",
            Duration::from_secs(2),
        )
        .await,
        "the oversized push should be metered"
    );
}

#[tokio::test]
async fn push_under_cap_is_204() {
    let state = capped_state();
    let addr = spawn_server(state).await;

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({"stream": "capped", "payload": {"a": 1}}))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn batch_with_one_oversized_event_rejects_whole_batch() {
    let state = capped_state();
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
            stream: "capped".into(),
            key: None,
        },
    )
    .await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("capped") >= 1
        })
        .await,
        "subscription did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({"events": [
            {"stream": "capped", "payload": {"first": true}},
            {"stream": "capped", "payload": oversized()}
        ]}))
        .send()
        .await
        .expect("batch");
    assert_eq!(resp.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);

    // All-or-nothing: the in-cap first event must NOT have been
    // dispatched. Push a sentinel and prove it arrives first.
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({"stream": "capped", "payload": {"sentinel": true}}))
        .send()
        .await
        .expect("sentinel push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    match recv_event(&mut ws).await {
        ServerFrame::Event { payload, .. } => {
            assert_eq!(
                payload,
                serde_json::json!({"sentinel": true}),
                "the rejected batch's in-cap event must not have been delivered"
            );
        }
        other => panic!("expected event, got {other:?}"),
    }
}

#[tokio::test]
async fn relay_receive_enforces_cap_defense_in_depth() {
    let state = capped_state();
    let addr = spawn_server(state).await;

    let body = cbor_batch(&serde_json::json!({
        "events": [{"stream": "capped", "payload": oversized()}]
    }));
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .body(body)
        .send()
        .await
        .expect("relay post");
    assert_eq!(resp.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
}
