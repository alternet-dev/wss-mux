use std::time::Duration;

use wss_mux::envelope::{ClientFrame, ServerFrame};

use crate::common::{
    connect_ws, poll_until, recv_event, send_frame, sign_token, spawn_server,
    test_state_with_manifest, Ws, PUSH_TOKEN,
};

async fn subscribe_chat_messages(ws: &mut Ws) {
    send_frame(
        ws,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;
    send_frame(
        ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;
}

#[tokio::test]
async fn dropping_ws_releases_subscriptions() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    {
        let mut ws = connect_ws(addr).await;
        subscribe_chat_messages(&mut ws).await;

        assert!(
            poll_until(Duration::from_secs(2), || {
                state.registry().binding_count("chat_messages") >= 1
            })
            .await,
            "subscription did not register"
        );
    }

    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") == 0
        })
        .await,
        "registry was not cleaned up after disconnect"
    );
}

#[tokio::test]
async fn reconnect_resubscribes_and_receives_new_events() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    {
        let mut ws = connect_ws(addr).await;
        subscribe_chat_messages(&mut ws).await;
        assert!(
            poll_until(Duration::from_secs(2), || {
                state.registry().binding_count("chat_messages") >= 1
            })
            .await,
            "first subscription did not register"
        );
    }

    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") == 0
        })
        .await,
        "registry not cleaned up before reconnect"
    );

    let mut ws = connect_ws(addr).await;
    subscribe_chat_messages(&mut ws).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "second subscription did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages",
            "payload": {"text": "after-reconnect"}
        }))
        .send()
        .await
        .expect("push");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    let ServerFrame::Event { id, payload, .. } = recv_event(&mut ws).await else {
        panic!("expected event after reconnect");
    };
    assert_eq!(id, "s1");
    assert_eq!(payload["text"], "after-reconnect");
}
