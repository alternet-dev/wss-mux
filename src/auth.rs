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

pub fn audience_admits(principals: &[String], audience: &[String]) -> bool {
    audience.iter().any(|a| principals.iter().any(|p| p == a))
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
}
