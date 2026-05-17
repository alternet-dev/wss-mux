use jsonwebtoken::{decode, errors::ErrorKind, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

pub fn validate_token(token: &str, signing_key: &str) -> Result<Claims, AuthError> {
    let key = DecodingKey::from_secret(signing_key.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["iss", "iat", "exp", "sub"]);
    validation.leeway = 0;

    match decode::<Claims>(token, &key, &validation) {
        Ok(data) => Ok(data.claims),
        Err(err) => match err.kind() {
            ErrorKind::ExpiredSignature => Err(AuthError::Expired),
            other => Err(AuthError::Invalid(format!("{other:?}"))),
        },
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
    use std::time::{SystemTime, UNIX_EPOCH};

    const KEY: &str = "secret-key";

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
        let parsed = validate_token(&token, KEY).expect("valid");
        assert_eq!(parsed, claims);
    }

    #[test]
    fn wrong_signing_key_is_invalid() {
        let token = sign(&fresh_claims(), KEY);
        let err = validate_token(&token, "other-key").expect_err("err");
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
        let err = validate_token(&token, KEY).expect_err("err");
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
        let err = validate_token(&token, KEY).expect_err("err");
        assert!(matches!(err, AuthError::Invalid(_)));
    }

    #[test]
    fn garbage_token_is_invalid() {
        let err = validate_token("not.a.jwt", KEY).expect_err("err");
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
