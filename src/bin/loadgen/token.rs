//! Subscriber-token minting for the harness.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use wss_mux::auth::Claims;

/// Mint an HS256 handshake token carrying `principals`, valid for an
/// hour — comfortably longer than any measurement window.
pub fn mint_token(signing_key: &str, principals: &[&str]) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs();
    let claims = Claims {
        iss: "wss-mux-loadgen".to_string(),
        iat: now,
        exp: now + 3600,
        sub: "loadgen".to_string(),
        principals: principals.iter().map(|p| p.to_string()).collect(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(signing_key.as_bytes()),
    )
    .context("encode handshake token")
}
