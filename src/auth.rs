use std::str::FromStr;

use jsonwebtoken::jwk::{AlgorithmParameters, Jwk, JwkSet};
use jsonwebtoken::{decode, decode_header, errors::ErrorKind, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::config::OidcConfig;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Claims {
    pub iss: String,
    pub iat: u64,
    pub exp: u64,
    pub sub: String,
    pub principals: Vec<String>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("expired token")]
    Expired,
    #[error("invalid token: {0}")]
    Invalid(String),
}

/// Decode `Claims` with the algorithm **pinned** to `alg` and verified
/// against `key`. The token's own `alg` header never widens what's
/// accepted — it only ever selects *which* configured key (see
/// [`HandshakeVerifier::validate`]), closing the JWT
/// algorithm-confusion hole.
fn decode_claims(token: &str, key: &DecodingKey, alg: Algorithm) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(alg);
    validation.set_required_spec_claims(&["iss", "iat", "exp", "sub"]);
    validation.leeway = 0;
    match decode::<Claims>(token, key, &validation) {
        Ok(data) => Ok(data.claims),
        Err(err) => match err.kind() {
            ErrorKind::ExpiredSignature => Err(AuthError::Expired),
            other => Err(AuthError::Invalid(format!("{other:?}"))),
        },
    }
}

/// Error building a [`HandshakeVerifier`] from configuration — surfaced
/// at startup so a misconfigured key fails loudly rather than silently
/// rejecting every connection.
#[derive(Debug, Error)]
pub enum VerifierBuildError {
    #[error(
        "no handshake key configured: set WSS_MUX_HANDSHAKE_SIGNING_KEY \
         and/or WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY[_FILE]"
    )]
    NoKey,
    #[error("invalid Ed25519 public key PEM: {0}")]
    Ed25519Pem(String),
}

/// Validates handshake tokens with the algorithm pinned to the
/// *configured* key. With both keys present the token's `alg` header
/// selects which configured key to check against — never which
/// algorithms are accepted — so an attacker who knows the Ed25519
/// public key cannot pass it off as an HS256 secret.
pub enum HandshakeVerifier {
    Hs256(DecodingKey),
    Ed25519(DecodingKey),
    Both {
        hs256: DecodingKey,
        ed25519: DecodingKey,
    },
}

impl HandshakeVerifier {
    /// At least one of HS256 / Ed25519 must be configured.
    pub fn new(
        hs256_secret: Option<&str>,
        ed25519_pem: Option<&str>,
    ) -> Result<Self, VerifierBuildError> {
        let hs = hs256_secret.map(|s| DecodingKey::from_secret(s.as_bytes()));
        let ed = match ed25519_pem {
            Some(pem) => Some(
                DecodingKey::from_ed_pem(pem.as_bytes())
                    .map_err(|e| VerifierBuildError::Ed25519Pem(format!("{:?}", e.kind())))?,
            ),
            None => None,
        };
        match (hs, ed) {
            (Some(hs256), Some(ed25519)) => Ok(Self::Both { hs256, ed25519 }),
            (Some(k), None) => Ok(Self::Hs256(k)),
            (None, Some(k)) => Ok(Self::Ed25519(k)),
            (None, None) => Err(VerifierBuildError::NoKey),
        }
    }

    pub fn validate(&self, token: &str) -> Result<Claims, AuthError> {
        let (key, alg) = match self {
            Self::Hs256(k) => (k, Algorithm::HS256),
            Self::Ed25519(k) => (k, Algorithm::EdDSA),
            Self::Both { hs256, ed25519 } => {
                let header = decode_header(token)
                    .map_err(|e| AuthError::Invalid(format!("{:?}", e.kind())))?;
                match header.alg {
                    Algorithm::HS256 => (hs256, Algorithm::HS256),
                    Algorithm::EdDSA => (ed25519, Algorithm::EdDSA),
                    other => return Err(AuthError::Invalid(format!("unsupported alg {other:?}"))),
                }
            }
        };
        decode_claims(token, key, alg)
    }
}

/// Validates an IdP-issued JWT against the cached JWKS. Mutually
/// exclusive with [`HandshakeVerifier`] (issuer XOR handshake). The
/// signing algorithm is taken from the JWK and `Validation` is locked
/// to it — the token's own `alg` header never widens what is accepted.
pub struct OidcVerifier {
    issuer: String,
    audience: String,
    groups_claim: String,
    principal_prefix: String,
}

impl OidcVerifier {
    pub fn from_config(cfg: &OidcConfig) -> Self {
        Self {
            issuer: cfg.issuer.clone(),
            audience: cfg.audience.clone(),
            groups_claim: cfg.groups_claim.clone(),
            principal_prefix: cfg.principal_prefix.clone(),
        }
    }

    pub fn validate(&self, token: &str, jwks: &JwkSet) -> Result<Claims, AuthError> {
        let header =
            decode_header(token).map_err(|e| AuthError::Invalid(format!("{:?}", e.kind())))?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::Invalid("token has no `kid`".into()))?;
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| AuthError::Invalid(format!("no JWK for kid `{kid}`")))?;
        let alg = jwk_alg(jwk)
            .ok_or_else(|| AuthError::Invalid("JWK has no usable signing algorithm".into()))?;
        let key = DecodingKey::from_jwk(jwk)
            .map_err(|e| AuthError::Invalid(format!("{:?}", e.kind())))?;

        let mut v = Validation::new(alg);
        v.set_issuer(&[&self.issuer]);
        v.set_audience(&[&self.audience]);
        v.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        v.validate_nbf = true;
        v.leeway = 0;
        let data = decode::<Value>(token, &key, &v).map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => AuthError::Expired,
            other => AuthError::Invalid(format!("{other:?}")),
        })?;
        Ok(map_oidc_claims(
            &data.claims,
            &self.groups_claim,
            &self.principal_prefix,
        ))
    }
}

/// JWT `alg` for a JWK: the JWK's declared algorithm if present, else
/// inferred from the key type. Returned `alg` is what `Validation` is
/// locked to (never the token's header `alg`).
fn jwk_alg(jwk: &Jwk) -> Option<Algorithm> {
    if let Some(ka) = jwk.common.key_algorithm {
        return Algorithm::from_str(&ka.to_string()).ok();
    }
    match &jwk.algorithm {
        AlgorithmParameters::RSA(_) => Some(Algorithm::RS256),
        AlgorithmParameters::EllipticCurve(_) => Some(Algorithm::ES256),
        AlgorithmParameters::OctetKeyPair(_) => Some(Algorithm::EdDSA),
        AlgorithmParameters::OctetKey(_) => Some(Algorithm::HS256),
    }
}

/// Map validated OIDC claims to the internal [`Claims`]. Principals are
/// always `user:<sub>` plus every value of the configured groups claim
/// with the configured prefix. A missing/empty/non-array groups claim
/// yields just the `user:` principal — a low-privilege set, **not** an
/// error (parity with a handshake token that carries no principals).
fn map_oidc_claims(v: &Value, groups_claim: &str, prefix: &str) -> Claims {
    let sub = v
        .get("sub")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut principals = vec![format!("user:{sub}")];
    if let Some(arr) = v.get(groups_claim).and_then(Value::as_array) {
        for g in arr {
            if let Some(s) = g.as_str() {
                principals.push(format!("{prefix}{s}"));
            }
        }
    }
    Claims {
        iss: v
            .get("iss")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        iat: v.get("iat").and_then(Value::as_u64).unwrap_or(0),
        exp: v.get("exp").and_then(Value::as_u64).unwrap_or(0),
        sub,
        principals,
    }
}

/// A connection's `principals` are admitted if they satisfy any single
/// audience entry. An entry is one of:
/// - `*` — public to *any* authenticated connection (matches even with
///   no principals; auth already happened at the connection level).
/// - `prefix*` — a trailing-`*` prefix match (e.g. `role:*` admits
///   `role:member`); still requires a matching principal.
/// - anything else — an exact match.
///
/// A `*` anywhere but a single trailing position is treated as a
/// literal here; the manifest rejects such entries at load time, so in
/// practice only the two wildcard forms above occur.
pub fn audience_admits(principals: &[String], audience: &[String]) -> bool {
    audience.iter().any(|entry| {
        if entry == "*" {
            true
        } else if let Some(prefix) = entry.strip_suffix('*') {
            principals.iter().any(|p| p.starts_with(prefix))
        } else {
            principals.iter().any(|p| p == entry)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde_json::json;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    const KEY: &str = "secret-key";

    // --- OIDC --------------------------------------------------------

    // HS256 oct JWK; `k` is base64url("secret").
    fn oidc_jwks() -> JwkSet {
        serde_json::from_str(r#"{"keys":[{"kty":"oct","kid":"k1","alg":"HS256","k":"c2VjcmV0"}]}"#)
            .expect("jwks")
    }

    fn oidc_verifier() -> OidcVerifier {
        OidcVerifier::from_config(&OidcConfig {
            issuer: "https://idp.example".into(),
            audience: "wss-mux".into(),
            jwks_url: None,
            groups_claim: "groups".into(),
            principal_prefix: "role:".into(),
            jwks_refresh: Duration::from_secs(300),
        })
    }

    fn sign_oidc(claims: Value, kid: Option<&str>) -> String {
        let mut h = Header::new(Algorithm::HS256);
        h.kid = kid.map(str::to_string);
        encode(&h, &claims, &EncodingKey::from_secret(b"secret")).unwrap()
    }

    fn oidc_claims(iss: &str, aud: &str, exp_offset: i64, groups: Option<Value>) -> Value {
        let n = now() as i64;
        let mut c = json!({
            "iss": iss, "aud": aud, "sub": "alice",
            "iat": n, "exp": n + exp_offset,
        });
        if let Some(g) = groups {
            c["groups"] = g;
        }
        c
    }

    #[test]
    fn oidc_valid_token_maps_user_and_prefixed_groups() {
        let v = oidc_verifier();
        let t = sign_oidc(
            oidc_claims(
                "https://idp.example",
                "wss-mux",
                300,
                Some(json!(["admins", "ops"])),
            ),
            Some("k1"),
        );
        let claims = v.validate(&t, &oidc_jwks()).expect("valid");
        assert_eq!(claims.sub, "alice");
        assert_eq!(
            claims.principals,
            vec![
                "user:alice".to_string(),
                "role:admins".to_string(),
                "role:ops".to_string()
            ]
        );
    }

    #[test]
    fn oidc_missing_groups_is_low_priv_not_error() {
        let v = oidc_verifier();
        let t = sign_oidc(
            oidc_claims("https://idp.example", "wss-mux", 300, None),
            Some("k1"),
        );
        let claims = v.validate(&t, &oidc_jwks()).expect("valid, low-priv");
        assert_eq!(claims.principals, vec!["user:alice".to_string()]);
    }

    #[test]
    fn oidc_wrong_audience_is_invalid() {
        let v = oidc_verifier();
        let t = sign_oidc(
            oidc_claims("https://idp.example", "someone-else", 300, None),
            Some("k1"),
        );
        assert!(matches!(
            v.validate(&t, &oidc_jwks()),
            Err(AuthError::Invalid(_))
        ));
    }

    #[test]
    fn oidc_wrong_issuer_is_invalid() {
        let v = oidc_verifier();
        let t = sign_oidc(
            oidc_claims("https://evil.example", "wss-mux", 300, None),
            Some("k1"),
        );
        assert!(matches!(
            v.validate(&t, &oidc_jwks()),
            Err(AuthError::Invalid(_))
        ));
    }

    #[test]
    fn oidc_expired_is_expired_variant() {
        let v = oidc_verifier();
        let t = sign_oidc(
            oidc_claims("https://idp.example", "wss-mux", -3600, None),
            Some("k1"),
        );
        assert!(matches!(
            v.validate(&t, &oidc_jwks()),
            Err(AuthError::Expired)
        ));
    }

    #[test]
    fn oidc_unknown_kid_is_invalid() {
        let v = oidc_verifier();
        let t = sign_oidc(
            oidc_claims("https://idp.example", "wss-mux", 300, None),
            Some("k2"),
        );
        assert!(matches!(
            v.validate(&t, &oidc_jwks()),
            Err(AuthError::Invalid(_))
        ));
    }

    #[test]
    fn oidc_no_kid_is_invalid() {
        let v = oidc_verifier();
        let t = sign_oidc(
            oidc_claims("https://idp.example", "wss-mux", 300, None),
            None,
        );
        assert!(matches!(
            v.validate(&t, &oidc_jwks()),
            Err(AuthError::Invalid(_))
        ));
    }

    #[test]
    fn oidc_tampered_signature_is_invalid() {
        let v = oidc_verifier();
        let mut t = sign_oidc(
            oidc_claims("https://idp.example", "wss-mux", 300, None),
            Some("k1"),
        );
        let last = t.pop().unwrap();
        t.push(if last == 'A' { 'B' } else { 'A' });
        assert!(matches!(
            v.validate(&t, &oidc_jwks()),
            Err(AuthError::Invalid(_))
        ));
    }

    // Deterministic Ed25519 test keypair (PKCS8 / SPKI PEM).
    const ED25519_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEINS+Ri6hNJgwRt84yvchqfGNA8ufVJ/7PlEI7O1RQPWe\n-----END PRIVATE KEY-----\n";
    const ED25519_PUB_PEM: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAGru6jfUFXaDDOSCuIvObd8KSbVpkQb43iORKVTKZuMw=\n-----END PUBLIC KEY-----\n";

    fn sign_ed25519(claims: &Claims, priv_pem: &str) -> String {
        encode(
            &Header::new(Algorithm::EdDSA),
            claims,
            &EncodingKey::from_ed_pem(priv_pem.as_bytes()).expect("ed priv pem"),
        )
        .unwrap()
    }

    #[test]
    fn hs256_only_verifier_accepts_hs256_rejects_eddsa() {
        let v = HandshakeVerifier::new(Some(KEY), None).expect("hs256 verifier");
        assert_eq!(
            v.validate(&sign(&fresh_claims(), KEY)).unwrap(),
            fresh_claims()
        );
        let ed = sign_ed25519(&fresh_claims(), ED25519_PRIV_PEM);
        assert!(matches!(v.validate(&ed), Err(AuthError::Invalid(_))));
    }

    #[test]
    fn ed25519_only_verifier_accepts_eddsa_rejects_hs256() {
        let v = HandshakeVerifier::new(None, Some(ED25519_PUB_PEM)).expect("ed verifier");
        let ed = sign_ed25519(&fresh_claims(), ED25519_PRIV_PEM);
        assert_eq!(v.validate(&ed).unwrap(), fresh_claims());
        let hs = sign(&fresh_claims(), KEY);
        assert!(matches!(v.validate(&hs), Err(AuthError::Invalid(_))));
    }

    #[test]
    fn both_verifier_dispatches_on_token_alg() {
        let v = HandshakeVerifier::new(Some(KEY), Some(ED25519_PUB_PEM)).expect("both");
        assert_eq!(
            v.validate(&sign(&fresh_claims(), KEY)).unwrap(),
            fresh_claims()
        );
        assert_eq!(
            v.validate(&sign_ed25519(&fresh_claims(), ED25519_PRIV_PEM))
                .unwrap(),
            fresh_claims()
        );
    }

    #[test]
    fn both_verifier_rejects_algorithm_confusion() {
        // Attacker knows the Ed25519 *public* key. They forge a token
        // with header alg=HS256, HMAC-signed using the public key PEM
        // as the shared secret. A naive verifier that trusts the
        // token's alg and reaches for "the key" would accept it. Ours
        // pins HS256 to the *configured HS256 secret*, so this MUST be
        // rejected.
        let v = HandshakeVerifier::new(Some(KEY), Some(ED25519_PUB_PEM)).expect("both");
        let forged = encode(
            &Header::new(Algorithm::HS256),
            &fresh_claims(),
            &EncodingKey::from_secret(ED25519_PUB_PEM.as_bytes()),
        )
        .unwrap();
        assert!(
            matches!(v.validate(&forged), Err(AuthError::Invalid(_))),
            "algorithm-confusion forgery must be rejected"
        );
    }

    #[test]
    fn verifier_requires_at_least_one_key() {
        assert!(matches!(
            HandshakeVerifier::new(None, None),
            Err(VerifierBuildError::NoKey)
        ));
    }

    #[test]
    fn verifier_rejects_malformed_ed25519_pem() {
        assert!(matches!(
            HandshakeVerifier::new(
                None,
                Some("-----BEGIN PUBLIC KEY-----\nnope\n-----END PUBLIC KEY-----\n")
            ),
            Err(VerifierBuildError::Ed25519Pem(_))
        ));
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn sign(claims: &Claims, key: &str) -> String {
        encode(
            &Header::new(Algorithm::HS256),
            claims,
            &EncodingKey::from_secret(key.as_bytes()),
        )
        .unwrap()
    }

    fn fresh_claims() -> Claims {
        let n = now();
        Claims {
            iss: "test-app".into(),
            iat: n,
            exp: n + 300,
            sub: "user:alice".into(),
            principals: vec!["role:member".into(), "user:alice".into()],
        }
    }

    #[test]
    fn valid_token_returns_claims() {
        let claims = fresh_claims();
        let token = sign(&claims, KEY);
        let parsed = HandshakeVerifier::new(Some(KEY), None)
            .unwrap()
            .validate(&token)
            .expect("valid");
        assert_eq!(parsed, claims);
    }

    #[test]
    fn wrong_signing_key_is_invalid() {
        let token = sign(&fresh_claims(), KEY);
        let err = HandshakeVerifier::new(Some("other-key"), None)
            .unwrap()
            .validate(&token)
            .expect_err("err");
        assert!(matches!(err, AuthError::Invalid(_)));
    }

    #[test]
    fn expired_token_is_expired_variant() {
        let n = now();
        let claims = Claims {
            iss: "test-app".into(),
            iat: n - 7200,
            exp: n - 3600,
            sub: "user:alice".into(),
            principals: vec!["role:member".into()],
        };
        let token = sign(&claims, KEY);
        let err = HandshakeVerifier::new(Some(KEY), None)
            .unwrap()
            .validate(&token)
            .expect_err("err");
        assert!(matches!(err, AuthError::Expired));
    }

    #[test]
    fn missing_required_claim_is_invalid() {
        #[derive(Serialize)]
        struct Partial {
            iss: String,
            iat: u64,
            exp: u64,
            // intentionally no `sub` or `principals`
        }
        let n = now();
        let partial = Partial {
            iss: "test-app".into(),
            iat: n,
            exp: n + 300,
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &partial,
            &EncodingKey::from_secret(KEY.as_bytes()),
        )
        .unwrap();
        let err = HandshakeVerifier::new(Some(KEY), None)
            .unwrap()
            .validate(&token)
            .expect_err("err");
        assert!(matches!(err, AuthError::Invalid(_)));
    }

    #[test]
    fn garbage_token_is_invalid() {
        let err = HandshakeVerifier::new(Some(KEY), None)
            .unwrap()
            .validate("not.a.jwt")
            .expect_err("err");
        assert!(matches!(err, AuthError::Invalid(_)));
    }

    #[test]
    fn audience_admits_intersecting_set() {
        let principals = vec!["role:member".into(), "tenant:t1".into()];
        let audience = vec!["role:member".into()];
        assert!(audience_admits(&principals, &audience));
    }

    #[test]
    fn audience_rejects_disjoint_set() {
        let principals = vec!["role:guest".into()];
        let audience = vec!["role:member".into()];
        assert!(!audience_admits(&principals, &audience));
    }

    #[test]
    fn audience_rejects_empty_principals() {
        let audience = vec!["role:member".into()];
        assert!(!audience_admits(&[], &audience));
    }

    #[test]
    fn audience_rejects_empty_audience() {
        let principals = vec!["role:member".into()];
        assert!(!audience_admits(&principals, &[]));
    }

    #[test]
    fn bare_star_admits_any_authenticated_connection() {
        // "public to any authenticated connection" — admits even when
        // the token carries no principals at all.
        assert!(audience_admits(&[], &["*".to_string()]));
        assert!(audience_admits(
            &["role:guest".to_string()],
            &["*".to_string()]
        ));
    }

    #[test]
    fn trailing_star_is_a_prefix_match() {
        let principals = vec!["role:member".to_string()];
        assert!(audience_admits(&principals, &["role:*".to_string()]));
        assert!(audience_admits(
            &["chat_room42".to_string()],
            &["chat_*".to_string()]
        ));
    }

    #[test]
    fn trailing_star_rejects_non_prefix_principals() {
        let principals = vec!["tenant:t1".to_string(), "user:bob".to_string()];
        assert!(!audience_admits(&principals, &["role:*".to_string()]));
    }

    #[test]
    fn trailing_star_does_not_admit_empty_principals() {
        // A prefix wildcard still needs a matching principal; only the
        // bare `*` is unconditional.
        assert!(!audience_admits(&[], &["role:*".to_string()]));
    }

    #[test]
    fn star_only_matches_as_a_trailing_wildcard_not_inside() {
        // A non-trailing `*` is treated as a literal (manifest validation
        // rejects such entries anyway); it must not match by prefix.
        let principals = vec!["role:member".to_string()];
        assert!(!audience_admits(&principals, &["ro*le".to_string()]));
        assert!(audience_admits(
            &["ro*le".to_string()],
            &["ro*le".to_string()]
        ));
    }

    #[test]
    fn mixed_audience_admits_on_any_entry() {
        let principals = vec!["role:member".to_string()];
        let audience = vec!["tenant:x".to_string(), "role:*".to_string()];
        assert!(audience_admits(&principals, &audience));
    }
}
