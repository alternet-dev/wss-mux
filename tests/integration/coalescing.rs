use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use wss_mux::manifest::Manifest;
use wss_mux::peers::{PeerUrl, Scheme};
use wss_mux::server::relay::{spawn_relay_flusher, InterestSnapshot, RelayBatch, StreamInterest};
use wss_mux::server::AppState;

use crate::common::{sample_manifest, spawn_server, test_config, PUSH_TOKEN};

fn peer(addr: SocketAddr) -> PeerUrl {
    PeerUrl {
        scheme: Scheme::Http,
        host: addr.ip().to_string(),
        port: addr.port(),
    }
}

/// A fake peer relay endpoint that counts POSTs and total events.
async fn fake_peer() -> (SocketAddr, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let posts = Arc::new(AtomicUsize::new(0));
    let events = Arc::new(AtomicUsize::new(0));
    let (p, e) = (posts.clone(), events.clone());
    let app = axum::Router::new().route(
        "/internal/relay",
        axum::routing::post(move |body: axum::body::Bytes| {
            let (p, e) = (p.clone(), e.clone());
            async move {
                let b: RelayBatch = ciborium::from_reader(body.as_ref()).unwrap();
                p.fetch_add(1, Ordering::SeqCst);
                e.fetch_add(b.events.len(), Ordering::SeqCst);
                axum::http::StatusCode::NO_CONTENT
            }
        }),
    );
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    (addr, posts, events)
}

/// Delays the first chunk so concurrent chunk requests would let the
/// second request arrive first. A sequential sender must retain input order.
async fn ordered_peer() -> (SocketAddr, Arc<Mutex<Vec<u64>>>) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let observed = received.clone();
    let app = axum::Router::new().route(
        "/internal/relay",
        axum::routing::post(move |body: axum::body::Bytes| {
            let observed = observed.clone();
            async move {
                let batch: RelayBatch = ciborium::from_reader(body.as_ref()).expect("relay CBOR");
                let number = batch.events[0].payload["n"].as_u64().expect("event number");
                if number == 0 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                observed.lock().expect("received lock").push(number);
                axum::http::StatusCode::NO_CONTENT
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let addr = listener.local_addr().expect("address");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });
    (addr, received)
}

#[tokio::test]
async fn coalescing_batches_many_pushes_into_few_posts() {
    let (peer_addr, posts, events) = fake_peer().await;

    let mut cfg = test_config();
    cfg.relay_coalesce_ms = 40;
    cfg.relay_coalesce_max_events = 100; // size cap not hit; window coalesces
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
    state.set_peers(vec![peer(peer_addr)]);
    spawn_relay_flusher(state.clone());

    let addr_a = spawn_server(state.clone()).await;
    let http = reqwest::Client::new();
    for i in 0..20 {
        let r = http
            .post(format!("http://{addr_a}/events"))
            .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
            .json(&serde_json::json!({
                "stream": "chat_messages", "key": "room-42",
                "payload": {"n": i}
            }))
            .send()
            .await
            .expect("push");
        assert_eq!(r.status(), reqwest::StatusCode::NO_CONTENT);
    }

    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(events.load(Ordering::SeqCst), 20, "all events relayed");
    let p = posts.load(Ordering::SeqCst);
    assert!(p < 20, "coalesced: {p} POSTs for 20 events (want « 20)");
}

#[tokio::test]
async fn coalescing_off_is_one_post_per_push() {
    let (peer_addr, posts, events) = fake_peer().await;

    // Default test_config has relay_coalesce_ms = 0 ⇒ direct path.
    let state = AppState::new(test_config());
    state.set_manifest(sample_manifest());
    state.set_peers(vec![peer(peer_addr)]);
    spawn_relay_flusher(state.clone()); // no-op (coalescing off)

    let addr_a = spawn_server(state.clone()).await;
    let http = reqwest::Client::new();
    for i in 0..5 {
        http.post(format!("http://{addr_a}/events"))
            .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
            .json(&serde_json::json!({
                "stream": "chat_messages", "key": "room-42",
                "payload": {"n": i}
            }))
            .send()
            .await
            .expect("push");
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(events.load(Ordering::SeqCst), 5);
    assert_eq!(
        posts.load(Ordering::SeqCst),
        5,
        "coalescing off ⇒ one POST per push (byte-identical to pre-v0.5)"
    );
}

#[tokio::test]
async fn coalescing_filters_each_peer_by_selective_interest() {
    let (wanted_addr, wanted_posts, wanted_events) = fake_peer().await;
    let (empty_addr, empty_posts, empty_events) = fake_peer().await;

    let mut cfg = test_config();
    cfg.relay_coalesce_ms = 30;
    cfg.relay_coalesce_max_events = 100;
    let state = AppState::new(cfg);
    state.set_manifest(
        Manifest::from_str(
            r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    selective_relay: true
"#,
            std::path::Path::new("test.yaml"),
        )
        .expect("manifest"),
    );
    let wanted = peer(wanted_addr);
    let empty = peer(empty_addr);
    state.set_peers(vec![wanted.clone(), empty.clone()]);
    state.set_peer_interest(
        &wanted,
        Some(InterestSnapshot {
            streams: vec![StreamInterest {
                stream: "chat_messages".into(),
                all_keys: false,
                keys: vec!["room-42".into()],
            }],
        }),
        &["chat_messages".into()],
    );
    state.set_peer_interest(
        &empty,
        Some(InterestSnapshot::default()),
        &["chat_messages".into()],
    );
    spawn_relay_flusher(state.clone());

    let addr = spawn_server(state).await;
    let http = reqwest::Client::new();
    for i in 0..5 {
        http.post(format!("http://{addr}/events"))
            .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
            .json(&serde_json::json!({
                "stream": "chat_messages", "key": "room-42", "payload": {"n": i}
            }))
            .send()
            .await
            .expect("push");
    }
    tokio::time::sleep(Duration::from_millis(250)).await;

    assert_eq!(wanted_events.load(Ordering::SeqCst), 5);
    assert!(wanted_posts.load(Ordering::SeqCst) >= 1);
    assert_eq!(empty_events.load(Ordering::SeqCst), 0);
    assert_eq!(empty_posts.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn coalesced_chunks_preserve_order_per_peer() {
    let (peer_addr, received) = ordered_peer().await;
    let mut cfg = test_config();
    cfg.relay_coalesce_ms = 10;
    cfg.relay_coalesce_max_events = 1;
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
    state.set_peers(vec![peer(peer_addr)]);
    spawn_relay_flusher(state.clone());

    let addr = spawn_server(state).await;
    let response = reqwest::Client::new()
        .post(format!("http://{addr}/events/batch"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({
            "events": [
                {"stream": "chat_messages", "payload": {"n": 0}},
                {"stream": "chat_messages", "payload": {"n": 1}}
            ]
        }))
        .send()
        .await
        .expect("batch push");
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if received.lock().expect("received lock").len() == 2 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("both chunks relayed");
    assert_eq!(*received.lock().expect("received lock"), vec![0, 1]);
}
