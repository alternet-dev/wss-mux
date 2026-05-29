//! End-to-end behaviour of the per-source WS publish rate limit
//! (`WSS_MUX_WS_PUBLISH_RATE` + friends).
//!
//! Source identification:
//!
//!   - JWT `sub` non-empty                        ⇒ source = `sub:<sub>`
//!   - JWT `sub` empty AND require_sub == false   ⇒ source = `conn:<id>`
//!   - JWT `sub` empty AND require_sub == true    ⇒ `unauthorized_publish`
//!
//! Two WS sessions with the same `sub` share one bucket; distinct
//! `sub`s are isolated.

use std::path::Path;
use std::time::Duration;

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::json;
use wss_mux::auth::Claims;
use wss_mux::config::Config;
use wss_mux::envelope::ClientFrame;
use wss_mux::manifest::Manifest;
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, recv_error_frame, recv_event, send_frame, spawn_server, Ws, PUSH_TOKEN, SIGNING_KEY,
};

const MANIFEST: &str = r#"
version: 1
streams:
  - stream: chat
    subscribe: [role:member]
    publish:   [role:member]
"#;

/// Build a test config with the per-source publish limit dialed in.
fn config_with_publish_limit(rate: u32, burst: u32, require_sub: bool) -> Config {
    let mut c = Config::new(
        PUSH_TOKEN.into(),
        wss_mux::config::HandshakeKeyConfig {
            hs256_secret: Some(SIGNING_KEY.into()),
            ed25519_public_pem: None,
        },
        "test.yaml".into(),
    );
    // Match the rest of the integration suite: per-connection inbound
    // limit off, so the new per-source limit is the only thing capping
    // publish frame rate.
    c.inbound_rate_per_sec = 0;
    c.inbound_burst = 0;
    c.ws_publish_rate_per_sec = rate;
    c.ws_publish_burst = burst;
    c.ws_publish_require_sub = require_sub;
    c
}

fn state_with_publish_limit(rate: u32, burst: u32, require_sub: bool) -> AppState {
    let state = AppState::new(config_with_publish_limit(rate, burst, require_sub));
    state.set_manifest(Manifest::from_str(MANIFEST, Path::new("t.yaml")).expect("manifest"));
    state
}

/// HS256-sign a token with the given `sub` (and one fixed principal so
/// the publish audience check admits it).
fn sign_token_with_sub(sub: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        iss: "test".into(),
        iat: now,
        exp: now + 300,
        sub: sub.into(),
        principals: vec!["role:member".to_string()],
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SIGNING_KEY.as_bytes()),
    )
    .unwrap()
}

async fn auth_with_sub(ws: &mut Ws, sub: &str) {
    send_frame(
        ws,
        &ClientFrame::Auth {
            token: sign_token_with_sub(sub),
        },
    )
    .await;
}

fn publish_frame(id: &str) -> ClientFrame {
    ClientFrame::Publish {
        id: id.into(),
        stream: "chat".into(),
        key: None,
        payload: json!({"x": 1}),
    }
}

#[tokio::test]
async fn flood_from_one_source_eventually_rate_limited() {
    // burst = 2 → exactly 2 publishes admitted, the 3rd rejected.
    // rate = 0 means no refill within the window we care about.
    let state = state_with_publish_limit(1, 2, false);
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    auth_with_sub(&mut ws, "user:alice").await;

    for _ in 0..2 {
        send_frame(&mut ws, &publish_frame("p")).await;
    }
    send_frame(&mut ws, &publish_frame("p3")).await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "rate_limited");
    assert_eq!(id.as_deref(), Some("p3"));
}

#[tokio::test]
async fn two_connections_sharing_sub_share_budget() {
    // Same `sub` from two WS sessions → one bucket. burst = 1 → only
    // one of the two combined publishes is admitted.
    let state = state_with_publish_limit(1, 1, false);
    let addr = spawn_server(state).await;

    let mut a = connect_ws(addr).await;
    auth_with_sub(&mut a, "user:alice").await;
    let mut b = connect_ws(addr).await;
    auth_with_sub(&mut b, "user:alice").await;

    // Subscribe A to `chat` so its own publish round-trips back as an
    // event frame — that round-trip is the sync barrier proving A's
    // publish was admitted and dispatched *before* B's frame is sent.
    // Without it, the two sends race and the test is flaky.
    send_frame(
        &mut a,
        &ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat".into(),
            key: None,
        },
    )
    .await;
    send_frame(&mut a, &publish_frame("a1")).await;
    let _ = recv_event(&mut a).await;

    // Session B should be rate-limited despite never publishing before.
    send_frame(&mut b, &publish_frame("b1")).await;
    let (code, id) = recv_error_frame(&mut b).await;
    assert_eq!(code, "rate_limited");
    assert_eq!(id.as_deref(), Some("b1"));
}

#[tokio::test]
async fn distinct_subs_have_isolated_budgets() {
    // Two distinct `sub`s → two buckets. burst = 1 each — each one
    // admits their first publish independently.
    let state = state_with_publish_limit(1, 1, false);
    let addr = spawn_server(state).await;

    let mut alice = connect_ws(addr).await;
    auth_with_sub(&mut alice, "user:alice").await;
    let mut bob = connect_ws(addr).await;
    auth_with_sub(&mut bob, "user:bob").await;

    send_frame(&mut alice, &publish_frame("a1")).await;
    send_frame(&mut bob, &publish_frame("b1")).await;

    // Bob's second publish hits the limit (his bucket capacity = 1).
    send_frame(&mut bob, &publish_frame("b2")).await;
    let (code, id) = recv_error_frame(&mut bob).await;
    assert_eq!(code, "rate_limited");
    assert_eq!(id.as_deref(), Some("b2"));

    // Alice's second publish hits her own bucket's limit too.
    send_frame(&mut alice, &publish_frame("a2")).await;
    let (code, id) = recv_error_frame(&mut alice).await;
    assert_eq!(code, "rate_limited");
    assert_eq!(id.as_deref(), Some("a2"));
}

#[tokio::test]
async fn rate_zero_means_no_limit_at_all() {
    // The "feature off" baseline: rate == 0 ⇒ no limiter, no rejection.
    let state = state_with_publish_limit(0, 0, false);
    let addr = spawn_server(state.clone()).await;

    let mut ws = connect_ws(addr).await;
    auth_with_sub(&mut ws, "user:alice").await;
    for _ in 0..32 {
        send_frame(&mut ws, &publish_frame("p")).await;
    }
    // We expect no `rate_limited` to come back. There may be other
    // frames pending (e.g. revoke from manifest refresh) but in this
    // test setup none should. Give it a beat then verify.
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(state.metrics().ws_publish_rate_limited.get(), 0);
}

#[tokio::test]
async fn empty_sub_falls_back_to_conn_id_when_require_sub_is_off() {
    // burst = 1: each connection gets exactly one admit before its own
    // `conn:<id>` bucket exhausts. With distinct conn_ids, two
    // sub-less connections do not share a budget.
    let state = state_with_publish_limit(1, 1, false);
    let addr = spawn_server(state).await;

    let mut a = connect_ws(addr).await;
    auth_with_sub(&mut a, "").await;
    let mut b = connect_ws(addr).await;
    auth_with_sub(&mut b, "").await;

    // Each first publish from each conn is admitted.
    send_frame(&mut a, &publish_frame("a1")).await;
    send_frame(&mut b, &publish_frame("b1")).await;

    // A's second publish hits its own conn:<id> bucket.
    send_frame(&mut a, &publish_frame("a2")).await;
    let (code, id) = recv_error_frame(&mut a).await;
    assert_eq!(code, "rate_limited");
    assert_eq!(id.as_deref(), Some("a2"));
}

#[tokio::test]
async fn empty_sub_is_unauthorized_publish_when_require_sub_is_on() {
    // burst = 1 — high enough to admit any publish, so the strict-mode
    // rejection is the only thing the publisher could see.
    let state = state_with_publish_limit(1, 1, true);
    let addr = spawn_server(state).await;

    let mut ws = connect_ws(addr).await;
    auth_with_sub(&mut ws, "").await;
    send_frame(&mut ws, &publish_frame("p1")).await;

    let (code, id) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthorized_publish");
    assert_eq!(id.as_deref(), Some("p1"));
}

#[tokio::test]
async fn rate_limited_metric_increments_on_rejection() {
    let state = state_with_publish_limit(1, 1, false);
    let addr = spawn_server(state.clone()).await;

    let mut ws = connect_ws(addr).await;
    auth_with_sub(&mut ws, "user:alice").await;

    send_frame(&mut ws, &publish_frame("p1")).await;
    send_frame(&mut ws, &publish_frame("p2")).await;
    // p2 is rejected — wait for the error frame to land.
    let _ = recv_error_frame(&mut ws).await;
    assert_eq!(state.metrics().ws_publish_rate_limited.get(), 1);
}
