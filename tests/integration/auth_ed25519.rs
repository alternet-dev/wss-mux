use std::time::Duration;

use wss_mux::config::HandshakeKeyConfig;
use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_close_code, recv_error_frame, recv_event, sample_manifest,
    send_frame, sign_token, sign_token_ed25519, spawn_server, test_config, ED25519_PUB_PEM,
    PUSH_TOKEN,
};

fn ed25519_only_state() -> AppState {
    let mut cfg = test_config();
    cfg.handshake_keys = HandshakeKeyConfig {
        hs256_secret: None,
        ed25519_public_pem: Some(ED25519_PUB_PEM.into()),
    };
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
    state
}

#[tokio::test]
async fn ed25519_token_authenticates_and_receives_event() {
    let state = ed25519_only_state();
    let addr = spawn_server(state.clone()).await;

    let mut ws = connect_ws(addr).await;
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign_token_ed25519(&["role:member"]),
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
        "Ed25519-authenticated subscription did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {"hi": true}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event { id, payload, .. } = recv_event(&mut ws).await else {
        panic!("expected event after Ed25519 auth");
    };
    assert_eq!(id, "s1");
    assert_eq!(payload["hi"], true);
}

#[tokio::test]
async fn hs256_token_rejected_when_only_ed25519_configured() {
    let state = ed25519_only_state();
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    // A perfectly well-formed HS256 token — but this server only trusts
    // Ed25519, so it must be rejected, not silently accepted.
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;

    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn both_keys_accept_hs256_and_ed25519_clients() {
    // Adding Ed25519 alongside HS256 must not break existing HS256
    // clients: a "Both" server authenticates either token type.
    let mut cfg = test_config();
    cfg.handshake_keys = HandshakeKeyConfig {
        hs256_secret: Some(crate::common::SIGNING_KEY.into()),
        ed25519_public_pem: Some(ED25519_PUB_PEM.into()),
    };
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
    let addr = spawn_server(state.clone()).await;

    for token in [
        sign_token(&["role:member"]),
        sign_token_ed25519(&["role:member"]),
    ] {
        let mut ws = connect_ws(addr).await;
        send_frame(&mut ws, &ClientFrame::Auth { token }).await;
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
        let resp = http
            .post(format!("http://{addr}/events"))
            .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
            .json(&serde_json::json!({"stream":"chat_messages","payload":{"ok":1}}))
            .send()
            .await
            .expect("push");
        assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);
        let ServerFrame::Event { id, .. } = recv_event(&mut ws).await else {
            panic!("expected event");
        };
        assert_eq!(id, "s1");
        // Drop ws before the next iteration so the binding clears.
        drop(ws);
        assert!(
            poll_until(Duration::from_secs(2), || {
                state.registry().binding_count("chat_messages") == 0
            })
            .await,
            "binding did not clear after disconnect"
        );
    }
}
