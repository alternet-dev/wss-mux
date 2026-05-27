use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use wss_mux::peers::{PeerUrl, Scheme};
use wss_mux::server::relay::{spawn_relay_flusher, RelayBatch};
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
