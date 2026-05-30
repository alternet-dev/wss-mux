//! OIDC JWKS fetch + discovery.
//!
//! Resolves the JWKS endpoint — the explicit `WSS_MUX_OIDC_JWKS_URL`
//! override, else OIDC discovery from
//! `<issuer>/.well-known/openid-configuration` — then fetches and
//! parses the [`JwkSet`]. Token validation against the set lives in
//! [`crate::auth`] (`OidcVerifier`). JSON is parsed with `serde_json`
//! over raw bytes so no `reqwest` `json` feature is needed.

use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;

use crate::config::OidcConfig;

/// Minimal async HTTP GET so JWKS fetch/discovery is unit-testable
/// with an injected fake — no network in tests.
pub trait HttpGet {
    fn get_bytes(
        &self,
        url: String,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<u8>>> + Send;
}

impl HttpGet for reqwest::Client {
    fn get_bytes(
        &self,
        url: String,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<u8>>> + Send {
        let client = self.clone();
        async move {
            let resp = client.get(&url).send().await?.error_for_status()?;
            Ok(resp.bytes().await?.to_vec())
        }
    }
}

#[derive(Deserialize)]
struct Discovery {
    jwks_uri: String,
}

/// The JWKS endpoint: the explicit `WSS_MUX_OIDC_JWKS_URL` override if
/// set, else discovered from `<discovery_url|issuer>`'s well-known
/// document.
///
/// `discovery_url` decouples the *reach* of OIDC discovery from the
/// *iss* claim — the BACKCHANNEL_DYNAMIC pattern: tokens carry the
/// public issuer (which `set_issuer` pins in auth) while discovery
/// targets an in-cluster endpoint that hands back jwks_uri pointing
/// at the same in-cluster host.
pub async fn resolve_jwks_url(http: &impl HttpGet, cfg: &OidcConfig) -> anyhow::Result<String> {
    if let Some(url) = &cfg.jwks_url {
        return Ok(url.clone());
    }
    let base = cfg.discovery_url.as_deref().unwrap_or(&cfg.issuer);
    let discovery_url = format!(
        "{}/.well-known/openid-configuration",
        base.trim_end_matches('/')
    );
    let body = http.get_bytes(discovery_url).await?;
    let disc: Discovery = serde_json::from_slice(&body)
        .map_err(|e| anyhow::anyhow!("OIDC discovery document parse failed: {e}"))?;
    Ok(disc.jwks_uri)
}

/// Resolve the JWKS endpoint and fetch + parse the key set.
pub async fn fetch_jwks(http: &impl HttpGet, cfg: &OidcConfig) -> anyhow::Result<JwkSet> {
    let url = resolve_jwks_url(http, cfg).await?;
    let body = http.get_bytes(url).await?;
    let set: JwkSet =
        serde_json::from_slice(&body).map_err(|e| anyhow::anyhow!("JWKS parse failed: {e}"))?;
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    fn cfg(issuer: &str, jwks_url: Option<&str>) -> OidcConfig {
        cfg_with_discovery(issuer, None, jwks_url)
    }

    fn cfg_with_discovery(
        issuer: &str,
        discovery_url: Option<&str>,
        jwks_url: Option<&str>,
    ) -> OidcConfig {
        OidcConfig {
            issuer: issuer.into(),
            audience: "wss-mux".into(),
            discovery_url: discovery_url.map(str::to_string),
            jwks_url: jwks_url.map(str::to_string),
            groups_claim: "groups".into(),
            principal_prefix: String::new(),
            jwks_refresh: Duration::from_secs(300),
        }
    }

    // Canned responses keyed by URL.
    struct Fake(HashMap<String, Vec<u8>>);
    impl HttpGet for Fake {
        fn get_bytes(
            &self,
            url: String,
        ) -> impl std::future::Future<Output = anyhow::Result<Vec<u8>>> + Send {
            let r = self
                .0
                .get(&url)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("no canned response for {url}"));
            async move { r }
        }
    }

    const JWKS: &str = r#"{"keys":[{"kty":"oct","kid":"k1","alg":"HS256","k":"c2VjcmV0"}]}"#;

    #[tokio::test]
    async fn explicit_jwks_url_skips_discovery() {
        let f = Fake(HashMap::from([(
            "https://idp.example/keys".to_string(),
            JWKS.as_bytes().to_vec(),
        )]));
        let c = cfg("https://idp.example", Some("https://idp.example/keys"));
        assert_eq!(
            resolve_jwks_url(&f, &c).await.unwrap(),
            "https://idp.example/keys"
        );
        let set = fetch_jwks(&f, &c).await.unwrap();
        assert_eq!(set.keys.len(), 1);
    }

    #[tokio::test]
    async fn discovery_resolves_jwks_uri_then_fetches() {
        let disc = r#"{"issuer":"https://idp.example","jwks_uri":"https://idp.example/jwks.json"}"#;
        let f = Fake(HashMap::from([
            (
                "https://idp.example/.well-known/openid-configuration".to_string(),
                disc.as_bytes().to_vec(),
            ),
            (
                "https://idp.example/jwks.json".to_string(),
                JWKS.as_bytes().to_vec(),
            ),
        ]));
        let c = cfg("https://idp.example/", None); // trailing slash trimmed
        assert_eq!(
            resolve_jwks_url(&f, &c).await.unwrap(),
            "https://idp.example/jwks.json"
        );
        assert_eq!(fetch_jwks(&f, &c).await.unwrap().keys.len(), 1);
    }

    #[tokio::test]
    async fn discovery_url_base_overrides_issuer() {
        // The Keycloak BACKCHANNEL_DYNAMIC pattern: tokens carry a
        // public issuer (`https://auth.example.com/realms/x`) while
        // discovery is reached via an in-cluster name
        // (`http://keycloak:8080/realms/x`). resolve_jwks_url must
        // fetch discovery from the in-cluster URL, not the public
        // one. The discovery document then dictates the JWKS URL.
        let disc = r#"{"issuer":"https://auth.example.com/realms/x","jwks_uri":"http://keycloak:8080/realms/x/protocol/openid-connect/certs"}"#;
        let f = Fake(HashMap::from([
            (
                "http://keycloak:8080/realms/x/.well-known/openid-configuration".to_string(),
                disc.as_bytes().to_vec(),
            ),
            (
                "http://keycloak:8080/realms/x/protocol/openid-connect/certs".to_string(),
                JWKS.as_bytes().to_vec(),
            ),
        ]));
        let c = cfg_with_discovery(
            "https://auth.example.com/realms/x",
            Some("http://keycloak:8080/realms/x"),
            None,
        );
        assert_eq!(
            resolve_jwks_url(&f, &c).await.unwrap(),
            "http://keycloak:8080/realms/x/protocol/openid-connect/certs"
        );
        assert_eq!(fetch_jwks(&f, &c).await.unwrap().keys.len(), 1);
    }

    #[tokio::test]
    async fn jwks_url_still_short_circuits_discovery_url() {
        // Explicit `jwks_url` overrides everything — discovery is
        // never reached, so the discovery_url field is irrelevant.
        let f = Fake(HashMap::from([(
            "https://idp.example/keys".to_string(),
            JWKS.as_bytes().to_vec(),
        )]));
        let c = cfg_with_discovery(
            "https://idp.example",
            Some("http://internal-but-unused"),
            Some("https://idp.example/keys"),
        );
        assert_eq!(
            resolve_jwks_url(&f, &c).await.unwrap(),
            "https://idp.example/keys"
        );
    }

    #[tokio::test]
    async fn fetch_failure_propagates() {
        let f = Fake(HashMap::new());
        let c = cfg("https://idp.example", Some("https://idp.example/keys"));
        assert!(fetch_jwks(&f, &c).await.is_err());
    }
}
