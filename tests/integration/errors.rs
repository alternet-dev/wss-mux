use tokio_tungstenite::tungstenite::Message as WsMessage;
use wss_mux::envelope::ClientFrame;

use crate::common::{
    connect_ws, recv_close_code, recv_error_frame, send_frame, send_text, sign_token,
    sign_token_with_offsets, spawn_server, test_state_with_manifest,
};

#[tokio::test]
async fn bad_frame_emits_error_and_closes_4400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    send_text(&mut ws, "not valid json".into()).await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "bad_frame");
    assert_eq!(id, None);
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn unknown_frame_type_emits_error_and_closes_4400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    send_text(&mut ws, r#"{"type":"nope"}"#.into()).await;

    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unknown_frame_type");
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn binary_frame_emits_bad_frame_and_closes_4400() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    use futures_util::SinkExt;
    ws.send(WsMessage::Binary(vec![0, 1, 2]))
        .await
        .expect("send");

    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "bad_frame");
    assert_eq!(recv_close_code(&mut ws).await, 4400);
}

#[tokio::test]
async fn subscribe_before_auth_emits_unauthenticated_and_closes_4401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(id.as_deref(), Some("s1"));
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn expired_token_emits_expired_and_closes_4401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    let token = sign_token_with_offsets(&["role:member"], -7200, -3600);
    send_frame(&mut ws, &ClientFrame::Auth { token }).await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "expired_token");
    assert_eq!(id, None);
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn invalid_signature_emits_unauthenticated_and_closes_4401() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJ4IiwiaWF0IjoxLCJleHAiOjk5OTk5OTk5OTksInN1YiI6IngiLCJwcmluY2lwYWxzIjpbXX0.aaaaaa";
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: token.into(),
        },
    )
    .await;

    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn subscribe_to_unknown_stream_emits_error_and_stays_open() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
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
            stream: "no-such-stream".into(),
            key: None,
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unknown_stream");
    assert_eq!(id.as_deref(), Some("s1"));

    // Connection still works: a valid subscribe afterward doesn't emit an error.
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s2".into(),
            stream: "chat_messages".into(),
            key: None,
        },
    )
    .await;
    send_frame(&mut ws, &ClientFrame::Unsubscribe { id: "s2".into() }).await;
}

#[tokio::test]
async fn unauthorized_subscribe_emits_error_and_stays_open() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let mut ws = connect_ws(addr).await;

    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign_token(&["role:guest"]),
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

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthorized_subscribe");
    assert_eq!(id.as_deref(), Some("s1"));
}

#[tokio::test]
async fn duplicate_subscription_id_emits_error_and_stays_open() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
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
    send_frame(
        &mut ws,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
        },
    )
    .await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "duplicate_subscription_id");
    assert_eq!(id.as_deref(), Some("s1"));
}
