use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;

use crate::config::Config;
use crate::dispatcher::dispatch;
use crate::envelope::{EnvelopePaths, EventEnvelope};
use crate::server::AppState;

pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics().encode();
    (
        [(
            header::CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )],
        body,
    )
}

fn envelope_paths(config: &Config) -> EnvelopePaths<'_> {
    EnvelopePaths {
        stream: &config.envelope_stream_path,
        key: &config.envelope_key_path,
        payload: &config.envelope_payload_path,
    }
}

pub async fn push_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<Value>, axum::extract::rejection::JsonRejection>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authenticate(&headers, &state.config().push_auth_token)?;

    let Json(body) = body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid JSON body"))?;
    let envelope = EventEnvelope::from_value(&body, envelope_paths(state.config()))
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid envelope"))?;

    let manifest = state
        .manifest()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "manifest not loaded"))?;
    if manifest.stream(&envelope.stream).is_none() {
        return Err((StatusCode::NOT_FOUND, "unknown stream"));
    }

    dispatch(&state, envelope);
    Ok(StatusCode::NO_CONTENT)
}

/// Batch push. The wrapper is always `{"events": [...]}`; each element is
/// interpreted with the configured envelope paths. All-or-nothing per
/// docs/roadmap.md: any element failing envelope extraction fails the
/// whole batch with 400; any referencing a stream not in the manifest
/// fails with 404. Otherwise all events are dispatched and the response is
/// 204 with no per-event status.
pub async fn push_batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<Value>, axum::extract::rejection::JsonRejection>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authenticate(&headers, &state.config().push_auth_token)?;

    let Json(body) = body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid JSON body"))?;
    let events = body
        .get("events")
        .and_then(Value::as_array)
        .ok_or((StatusCode::BAD_REQUEST, "batch body needs an events array"))?;

    let manifest = state
        .manifest()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "manifest not loaded"))?;
    let paths = envelope_paths(state.config());

    let mut parsed = Vec::with_capacity(events.len());
    for raw in events {
        let envelope = EventEnvelope::from_value(raw, paths)
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid event in batch"))?;
        if manifest.stream(&envelope.stream).is_none() {
            return Err((StatusCode::NOT_FOUND, "unknown stream in batch"));
        }
        parsed.push(envelope);
    }

    for envelope in parsed {
        dispatch(&state, envelope);
    }
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
