use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{routing::get, Router};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use tokio::net::TcpListener;
use wss_mux::config::{HandshakeKeyConfig, OidcConfig};
use wss_mux::envelope::{ClientFrame, ServerFrame};
use wss_mux::server::AppState;

use crate::common::{
    connect_ws, poll_until, recv_close_code, recv_error_frame, recv_event, sample_manifest,
    send_frame, spawn_server, test_config, PUSH_TOKEN,
};

// HS256 oct JWK; `k` is base64url("secret"). Tokens are signed with the
// same secret + kid so a real signature verification path is exercised.
const JWKS: &str = r#"{"keys":[{"kty":"oct","kid":"k1","alg":"HS256","k":"c2VjcmV0"}]}"#;

/// Minimal in-test OIDC IdP: a discovery document + a JWKS endpoint.
/// Returns the issuer base URL.
async fn spawn_idp() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}", listener.local_addr().unwrap());
    let disc = format!(r#"{{"issuer":"{issuer}","jwks_uri":"{issuer}/jwks"}}"#);
    let app = Router::new()
        .route(
            "/.well-known/openid-configuration",
            get(move || {
                let d = disc.clone();
                async move { d }
            }),
        )
        .route("/jwks", get(|| async { JWKS }));
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    issuer
}

fn oidc_cfg(issuer: &str) -> OidcConfig {
    OidcConfig {
        issuer: issuer.to_string(),
        audience: "wss-mux".into(),
        jwks_url: None, // exercise discovery
        groups_claim: "groups".into(),
        principal_prefix: "role:".into(),
        jwks_refresh: Duration::from_secs(300),
    }
}

/// An OIDC-configured wss-mux state with the JWKS already fetched via
/// discovery (the refresher task only runs in the binary, not tests).
async fn ready_state(issuer: &str) -> AppState {
    let oidc = oidc_cfg(issuer);
    let mut cfg = test_config();
    cfg.handshake_keys = HandshakeKeyConfig {
        hs256_secret: None,
        ed25519_public_pem: None,
    };
    cfg.oidc = Some(oidc.clone());
    let state = AppState::new(cfg);
    state.set_manifest(sample_manifest());
    let jwks = wss_mux::oidc::fetch_jwks(&reqwest::Client::new(), &oidc)
        .await
        .expect("JWKS fetched via discovery");
    state.set_jwks(jwks);
    state
}

fn sign(issuer: &str, aud: &str, exp_off: i64, groups: &[&str], kid: &str) -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = serde_json::json!({
        "iss": issuer, "aud": aud, "sub": "alice",
        "iat": n, "exp": n + exp_off, "groups": groups,
    });
    let mut h = Header::new(Algorithm::HS256);
    h.kid = Some(kid.into());
    encode(&h, &claims, &EncodingKey::from_secret(b"secret")).unwrap()
}

#[tokio::test]
async fn oidc_token_authenticates_maps_groups_and_receives_event() {
    let issuer = spawn_idp().await;
    let state = ready_state(&issuer).await;
    let addr = spawn_server(state.clone()).await;

    let mut ws = connect_ws(addr).await;
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign(&issuer, "wss-mux", 300, &["member"], "k1"),
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
    // groups ["member"] + prefix "role:" → principal "role:member",
    // which the sample manifest's chat_messages audience admits.
    assert!(
        poll_until(Duration::from_secs(2), || {
            state.registry().binding_count("chat_messages") >= 1
        })
        .await,
        "OIDC-mapped principal should admit the subscription"
    );

    let http = reqwest::Client::new();
    let r = http
        .post(format!("http://{addr}/events"))
        .header("Authorization", format!("Bearer {PUSH_TOKEN}"))
        .json(&serde_json::json!({"stream":"chat_messages","payload":{"hi":true}}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), reqwest::StatusCode::NO_CONTENT);
    let ServerFrame::Event { id, .. } = recv_event(&mut ws).await else {
        panic!("expected event after OIDC auth");
    };
    assert_eq!(id, "s1");
}

#[tokio::test]
async fn oidc_wrong_audience_is_unauthenticated_close() {
    let issuer = spawn_idp().await;
    let addr = spawn_server(ready_state(&issuer).await).await;
    let mut ws = connect_ws(addr).await;
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign(&issuer, "not-us", 300, &[], "k1"),
        },
    )
    .await;
    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn oidc_expired_token_is_expired_close() {
    let issuer = spawn_idp().await;
    let addr = spawn_server(ready_state(&issuer).await).await;
    let mut ws = connect_ws(addr).await;
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign(&issuer, "wss-mux", -3600, &[], "k1"),
        },
    )
    .await;
    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "expired_token");
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}

#[tokio::test]
async fn oidc_unknown_kid_is_unauthenticated_close() {
    let issuer = spawn_idp().await;
    let addr = spawn_server(ready_state(&issuer).await).await;
    let mut ws = connect_ws(addr).await;
    send_frame(
        &mut ws,
        &ClientFrame::Auth {
            token: sign(&issuer, "wss-mux", 300, &[], "unknown-kid"),
        },
    )
    .await;
    let (code, _) = recv_error_frame(&mut ws).await;
    assert_eq!(code, "unauthenticated");
    assert_eq!(recv_close_code(&mut ws).await, 4401);
}
