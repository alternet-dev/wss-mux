use std::net::SocketAddr;
use std::time::Duration;

use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::peers::{PeerUrl, Scheme};

use crate::common::{
    connect_ws, poll_metrics_for, poll_until, recv_event, send_frame, sign_token, spawn_server,
    test_state, test_state_with_manifest, Ws, PUSH_TOKEN,
};

/// A loopback `PeerUrl` pointing at a spawned test server.
fn peer(addr: SocketAddr) -> PeerUrl {
    PeerUrl {
        scheme: Scheme::Http,
        host: addr.ip().to_string(),
        port: addr.port(),
    }
}

/// Asserts no further event/data frame arrives within `dur` (ping/pong
/// ignored). Used to prove the one-hop guard — a relayed event must not
/// bounce back as a duplicate.
async fn assert_no_more_events(ws: &mut Ws, dur: Duration) {
    match tokio::time::timeout(dur, async {
        loop {
            match ws.next().await {
                Some(Ok(WsMessage::Ping(_))) | Some(Ok(WsMessage::Pong(_))) => continue,
                other => return other,
            }
        }
    })
    .await
    {
        Err(_) => {} // timed out with nothing — correct
        Ok(other) => panic!("expected no further events, got {other:?}"),
    }
}

/// A port with nothing listening: bind then drop, so connections are
/// refused fast (best-effort relay failure path, no timeout wait).
async fn dead_peer() -> PeerUrl {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    drop(l);
    peer(addr)
}

/// CBOR-encode a `{"events":[...]}` batch the way a peer (or PR4's
/// producer-side relay) will: serialize canonical envelopes, no
/// envelope-path coupling.
fn cbor_batch(value: &serde_json::Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf).expect("cbor encode");
    buf
}

#[tokio::test]
async fn relay_receive_delivers_to_local_subscribers() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state.clone()).await;

    let mut client = connect_ws(addr).await;
    send_frame(
        &mut client,
        &ClientFrame::Auth {
            token: sign_token(&["role:member"]),
        },
    )
    .await;
    send_frame(
        &mut client,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
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

    let body = cbor_batch(&serde_json::json!({
        "events": [
            {"stream": "chat_messages", "key": "room-42",
             "payload": {"from": "peer", "text": "relayed"}}
        ]
    }));
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .header("Content-Type", "application/cbor")
        .body(body)
        .send()
        .await
        .expect("relay post");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    match recv_event(&mut client).await {
        ServerFrame::Event {
            stream,
            key,
            payload,
            ..
        } => {
            assert_eq!(stream, "chat_messages");
            assert_eq!(key.as_deref(), Some("room-42"));
            assert_eq!(
                payload,
                serde_json::json!({"from": "peer", "text": "relayed"})
            );
        }
        other => panic!("expected event frame, got {other:?}"),
    }
}

#[tokio::test]
async fn relay_receive_requires_push_token() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let body = cbor_batch(&serde_json::json!({"events": []}));
    let http = reqwest::Client::new();

    let missing = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .body(body.clone())
        .send()
        .await
        .expect("post");
    assert_eq!(missing.status(), reqwest::StatusCode::UNAUTHORIZED);

    let wrong = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", "Bearer not-the-token")
        .body(body)
        .send()
        .await
        .expect("post");
    assert_eq!(wrong.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn relay_receive_rejects_invalid_cbor() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .body(vec![0xff, 0x00, 0x13, 0x37])
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn relay_receive_unknown_stream_is_404_all_or_nothing() {
    let state = test_state_with_manifest();
    let addr = spawn_server(state).await;
    let body = cbor_batch(&serde_json::json!({
        "events": [
            {"stream": "chat_messages", "payload": {}},
            {"stream": "no_such_stream", "payload": {}}
        ]
    }));
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .body(body)
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn relay_receive_without_manifest_returns_503() {
    let state = test_state(); // server up, manifest never set
    let addr = spawn_server(state).await;
    let body = cbor_batch(&serde_json::json!({
        "events": [{"stream": "chat_messages", "payload": {}}]
    }));
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr}/internal/v1/relay"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .body(body)
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
}

// ---- Outbound relay (the keystone) ------------------------------

async fn subscribe(addr: SocketAddr) -> Ws {
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
            key: Some("room-42".into()),
        },
    )
    .await;
    ws
}

#[tokio::test]
async fn producer_push_relays_to_peer_instance() {
    let state_a = test_state_with_manifest();
    let state_b = test_state_with_manifest();
    let addr_a = spawn_server(state_a.clone()).await;
    let addr_b = spawn_server(state_b.clone()).await;

    // A relays to B; B has no peers (default).
    state_a.set_peers(vec![peer(addr_b)]);

    let mut client_b = subscribe(addr_b).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state_b.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "subscription on B did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr_a}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages", "key": "room-42",
            "payload": {"text": "via-relay"}
        }))
        .send()
        .await
        .expect("push to A");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    match recv_event(&mut client_b).await {
        ServerFrame::Event {
            stream, payload, ..
        } => {
            assert_eq!(stream, "chat_messages");
            assert_eq!(payload, serde_json::json!({"text": "via-relay"}));
        }
        other => panic!("expected relayed event on B, got {other:?}"),
    }

    assert!(
        poll_metrics_for(addr_a, "wss_mux_relay_sent_total 1", Duration::from_secs(2)).await,
        "A should have metered exactly one successful relay"
    );
}

#[tokio::test]
async fn relay_receipt_is_not_re_relayed_one_hop_guard() {
    let state_a = test_state_with_manifest();
    let state_b = test_state_with_manifest();
    let addr_a = spawn_server(state_a.clone()).await;
    let addr_b = spawn_server(state_b.clone()).await;

    // Cross-pointed: if B re-relayed peer receipts, the event would loop
    // back to A (and forever). The one-hop guard must stop it.
    state_a.set_peers(vec![peer(addr_b)]);
    state_b.set_peers(vec![peer(addr_a)]);

    let mut client_a = subscribe(addr_a).await;
    let mut client_b = subscribe(addr_b).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state_a.registry().binding_count("chat_messages") >= 1
                && state_b.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "subscriptions did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr_a}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages", "key": "room-42",
            "payload": {"n": 1}
        }))
        .send()
        .await
        .expect("push to A");
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    // Each side gets exactly one copy: A locally, B via the single relay.
    for client in [&mut client_a, &mut client_b] {
        match recv_event(client).await {
            ServerFrame::Event { payload, .. } => {
                assert_eq!(payload, serde_json::json!({"n": 1}));
            }
            other => panic!("expected one event, got {other:?}"),
        }
    }
    // No bounce-back: B did not re-relay to A, A did not re-relay its
    // own /internal receipt, etc.
    assert_no_more_events(&mut client_a, Duration::from_millis(400)).await;
    assert_no_more_events(&mut client_b, Duration::from_millis(400)).await;
}

#[tokio::test]
async fn relay_failure_is_best_effort_and_metered() {
    let state_a = test_state_with_manifest();
    let addr_a = spawn_server(state_a.clone()).await;
    state_a.set_peers(vec![dead_peer().await]);

    let mut client_a = subscribe(addr_a).await;
    assert!(
        poll_until(Duration::from_secs(2), || {
            state_a.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "subscription did not register"
    );

    let http = reqwest::Client::new();
    let resp = http
        .post(format!("http://{addr_a}/v1/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "stream": "chat_messages", "key": "room-42",
            "payload": {"ok": true}
        }))
        .send()
        .await
        .expect("push to A");
    // Producer never sees backpressure from a dead peer.
    assert_eq!(resp.status(), reqwest::StatusCode::NO_CONTENT);

    // Local delivery is unaffected by the relay failure.
    match recv_event(&mut client_a).await {
        ServerFrame::Event { payload, .. } => {
            assert_eq!(payload, serde_json::json!({"ok": true}));
        }
        other => panic!("expected local event, got {other:?}"),
    }

    assert!(
        poll_metrics_for(addr_a, "wss_mux_relay_failed_total", Duration::from_secs(3)).await,
        "the failed relay should be metered"
    );
}
