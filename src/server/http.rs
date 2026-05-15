use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::Json;

use crate::dispatcher::dispatch;
use crate::envelope::EventEnvelope;
use crate::server::AppState;

pub async fn push_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<EventEnvelope>, axum::extract::rejection::JsonRejection>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authenticate(&headers, &state.config().push_auth_token)?;

    let Json(envelope) = body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid envelope"))?;

    if state
        .manifest()
        .and_then(|m| m.stream(&envelope.stream))
        .is_none()
    {
        return Err((StatusCode::NOT_FOUND, "unknown stream"));
    }

    dispatch(&state, envelope);
    Ok(StatusCode::NO_CONTENT)
}

fn authenticate(headers: &HeaderMap, expected: &str) -> Result<(), (StatusCode, &'static str)> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "missing authorization header"))?;
    let token = value
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "expected bearer token"))?;
    if subtle_eq(token.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err((StatusCode::UNAUTHORIZED, "invalid push token"))
    }
}

/// Constant-time byte slice comparison to avoid timing leaks on the push token.
fn subtle_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtle_eq_matches_equal_bytes() {
        assert!(subtle_eq(b"abc", b"abc"));
    }

    #[test]
    fn subtle_eq_rejects_different_bytes() {
        assert!(!subtle_eq(b"abc", b"abd"));
    }

    #[test]
    fn subtle_eq_rejects_different_length() {
        assert!(!subtle_eq(b"abc", b"abcd"));
    }
}
