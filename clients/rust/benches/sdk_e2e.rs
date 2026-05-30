//! End-to-end benches: SDK ↔ in-process wss-mux server.
//!
//! Each bench spawns a single wss-mux server bound to an ephemeral
//! port, then drives the SDK against it. Numbers are the full
//! roundtrip cost (SDK serialize → WS write → server parse + dispatch
//! → WS read → SDK deserialize → channel hand-off), not synthetic.
//!
//! Run:
//!
//! ```bash
//! cd clients/rust
//! cargo bench --bench sdk_e2e
//! ```
//!
//! `wss-mux`, `axum`, `jsonwebtoken`, and `criterion` are all dev-deps;
//! the published SDK's dependency graph is unaffected.

use std::net::SocketAddr;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

use wss_mux::config::{Config, HandshakeKeyConfig};
use wss_mux::manifest::Manifest;
use wss_mux::server::{build_app, AppState};
use wss_mux_client::{ReconnectOptions, Subscription, WssMuxClient};

const SIGNING_KEY: &str = "bench-handshake-secret";
const STREAM: &str = "bench";

const MANIFEST_YAML: &str = r#"
version: 1
streams:
  - stream: bench
    subscribe: ["role:member"]
    publish:   ["role:member"]
"#;

#[derive(serde::Serialize)]
struct Claims {
    iss: String,
    iat: u64,
    exp: u64,
    sub: String,
    principals: Vec<String>,
}

fn sign_token(principals: &[&str]) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        iss: "bench".into(),
        iat: now,
        exp: now + 3600,
        sub: "user:bench".into(),
        principals: principals.iter().map(|s| (*s).to_string()).collect(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SIGNING_KEY.as_bytes()),
    )
    .expect("encode token")
}

fn test_config() -> Config {
    // The bench process owns the env, so `Config::from_env` would be
    // hostile (it expects env vars). `Config::new` exists for embedders
    // and gives us every optional knob at its documented default.
    let mut cfg = Config::new(
        "bench-push-secret".into(),
        HandshakeKeyConfig {
            hs256_secret: Some(SIGNING_KEY.into()),
            ed25519_public_pem: None,
        },
        Path::new("bench.yaml").to_path_buf(),
    );
    // Disable per-connection inbound rate-limit; the bench needs to
    // fire publishes faster than the production default's 50/sec.
    cfg.inbound_rate_per_sec = 0;
    cfg.inbound_burst = 0;
    // Same for WS-publish per-source rate limit. The bench is a single
    // principal hammering one stream — that's the throughput surface
    // we want to measure, not the limiter behavior.
    cfg.ws_publish_rate_per_sec = 0;
    cfg.ws_publish_burst = 0;
    cfg
}

async fn spawn_server() -> SocketAddr {
    let state = AppState::new(test_config());
    state.set_manifest(
        Manifest::from_str(MANIFEST_YAML, Path::new("bench.yaml")).expect("parse manifest"),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, build_app(state)).await.ok();
    });
    addr
}

async fn make_client(addr: SocketAddr) -> WssMuxClient {
    let token = sign_token(&["role:member"]);
    WssMuxClient::builder()
        .url(format!("ws://{addr}/stream"))
        .get_token(move || {
            let t = token.clone();
            async move { Ok(t) }
        })
        // Settle 0 → publish() resolves immediately; the bench's recv()
        // is what catches the actual server roundtrip latency.
        .publish_settle(Duration::ZERO)
        .reconnect(ReconnectOptions {
            max_attempts: Some(0),
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(50),
            backoff_multiplier: 2.0,
        })
        .build()
        .await
        .expect("build")
}

/// Single publish + receive own event. Measures the full SDK ↔ server
/// roundtrip on loopback under a current-thread tokio runtime.
fn bench_publish_self_roundtrip(c: &mut Criterion) {
    let rt = Runtime::new().expect("runtime");
    let (client, sub) = rt.block_on(async {
        let addr = spawn_server().await;
        let client = make_client(addr).await;
        let sub = client
            .subscribe(STREAM, Some("rt"))
            .await
            .expect("subscribe");
        // Brief settle so the server's registry has the binding before
        // the first publish goes out.
        tokio::time::sleep(Duration::from_millis(50)).await;
        (client, sub)
    });
    // The Subscription is held inside a Mutex because criterion's
    // async iter closure runs many times; we need mutable access on
    // each iter to call recv().
    let sub = tokio::sync::Mutex::new(sub);

    c.bench_function("publish_self_roundtrip", |b| {
        b.to_async(&rt).iter(|| async {
            client
                .publish(STREAM, Some("rt"), json!({"x": 1}))
                .await
                .expect("publish");
            let mut guard = sub.lock().await;
            let item = guard.recv().await.expect("channel closed");
            item.expect("event");
        });
    });

    rt.block_on(async {
        drop(sub);
        let _ = client.close().await;
    });
}

/// Throughput: publish N events in a tight loop, then receive all N.
/// Reports events/second so the criterion summary line includes a
/// useful headline number.
fn bench_publish_throughput(c: &mut Criterion) {
    let rt = Runtime::new().expect("runtime");
    let (client, sub) = rt.block_on(async {
        let addr = spawn_server().await;
        let client = make_client(addr).await;
        let sub = client
            .subscribe(STREAM, Some("tp"))
            .await
            .expect("subscribe");
        tokio::time::sleep(Duration::from_millis(50)).await;
        (client, sub)
    });
    let sub = tokio::sync::Mutex::new(sub);

    let mut group = c.benchmark_group("publish_throughput");
    // Keep sample sizes modest so the full bench suite fits in ~30s.
    // For rigorous numbers pass `--measurement-time 30` on the CLI.
    group.sample_size(20);
    for &n in &[10u64, 100u64] {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.to_async(&rt).iter(|| async {
                for i in 0..n {
                    client
                        .publish(STREAM, Some("tp"), json!({"i": i}))
                        .await
                        .expect("publish");
                }
                let mut guard = sub.lock().await;
                for _ in 0..n {
                    let item = guard.recv().await.expect("channel closed");
                    item.expect("event");
                }
            });
        });
    }
    group.finish();

    rt.block_on(async {
        drop(sub);
        let _ = client.close().await;
    });
}

/// Subscribe-only after the connection is already established. Isolates
/// the round-trip cost of a single `subscribe` frame from connect
/// overhead. (Drop sends Unsubscribe, so each iteration includes one
/// of each.)
fn bench_subscribe(c: &mut Criterion) {
    let rt = Runtime::new().expect("runtime");
    let (addr, client) = rt.block_on(async {
        let addr = spawn_server().await;
        let client = make_client(addr).await;
        (addr, client)
    });
    let _ = addr;

    c.bench_function("subscribe_one", |b| {
        b.to_async(&rt).iter(|| async {
            let _sub: Subscription = client
                .subscribe(STREAM, Some("sub-bench"))
                .await
                .expect("subscribe");
            // Drop here → unsubscribe over the wire on next iter.
        });
    });

    rt.block_on(async {
        let _ = client.close().await;
    });
}

criterion_group!(
    benches,
    bench_publish_self_roundtrip,
    bench_publish_throughput,
    bench_subscribe
);
criterion_main!(benches);
