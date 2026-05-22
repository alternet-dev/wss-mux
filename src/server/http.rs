use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;

use crate::config::Config;
use crate::dispatcher::{dispatch, EventOrigin};
use crate::envelope::{EnvelopePaths, EventEnvelope};
use crate::server::metrics::{RejectReason, RejectReasonLabel};
use crate::server::relay::{spawn_relay, RelayBatch};
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

/// Shared ingest core for every event entry point. Validates each
/// envelope's stream against the current manifest (all-or-nothing),
/// dispatches locally, and — only for a producer origin — spawns a
/// best-effort relay of the canonical batch to the discovered peers.
/// A peer relay receipt is never re-relayed: the distinct entry path
/// *is* the structural one-hop loop guard.
fn accept_events(
    state: &AppState,
    events: Vec<EventEnvelope>,
    origin: EventOrigin,
) -> Result<(), (StatusCode, &'static str)> {
    let manifest = state
        .manifest()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "manifest not loaded"))?;
    // All-or-nothing validation: every envelope must reference a known
    // stream and fit that stream's payload cap before anything is
    // dispatched or relayed. The cap is measured as JSON-serialized
    // bytes so it is stable regardless of the producer/relay wire
    // codec, and is enforced on every origin (defense-in-depth across a
    // rolling deploy with mixed manifests).
    for envelope in &events {
        let Some(stream) = manifest.stream(&envelope.stream) else {
            return Err((StatusCode::NOT_FOUND, "unknown stream"));
        };
        if let Some(cap) = stream.max_payload_bytes {
            let size = serde_json::to_vec(&envelope.payload)
                .map(|b| b.len() as u64)
                .unwrap_or(0);
            if size > cap {
                state
                    .metrics()
                    .events_rejected
                    .get_or_create(&RejectReasonLabel {
                        reason: RejectReason::PayloadTooLarge,
                    })
                    .inc();
                return Err((StatusCode::PAYLOAD_TOO_LARGE, "payload too large"));
            }
        }
    }

    // Capture the canonical batch for relay before the dispatch loop
    // consumes it (only when this is a producer push).
    let relay_batch = (origin == EventOrigin::Producer).then(|| RelayBatch {
        events: events.clone(),
    });

    for envelope in events {
        dispatch(state, envelope, origin);
    }

    if let Some(batch) = relay_batch {
        spawn_relay(state, batch);
    }
    Ok(())
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

    accept_events(&state, vec![envelope], EventOrigin::Producer)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Batch push. The wrapper is always `{"events": [...]}`; each element is
/// interpreted with the configured envelope paths. All-or-nothing: any
/// element failing envelope extraction fails the whole batch with 400;
/// any referencing a stream not in the manifest fails with 404.
/// Otherwise all events are dispatched and the response is 204 with no
/// per-event status.
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

    let paths = envelope_paths(state.config());
    let mut parsed = Vec::with_capacity(events.len());
    for raw in events {
        let envelope = EventEnvelope::from_value(raw, paths)
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid event in batch"))?;
        parsed.push(envelope);
    }

    accept_events(&state, parsed, EventOrigin::Producer)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Peer relay receipt. The body is a CBOR-encoded [`RelayBatch`] of
/// already-canonical envelopes (no `WSS_MUX_ENVELOPE_*` coupling).
/// Authenticated with the same `WSS_MUX_PUSH_AUTH_TOKEN` as the producer
/// endpoints — both inject events, identical security boundary. Routes
/// straight into `accept_events`, which never re-relays: the distinct
/// path *is* the one-hop loop guard, not a spoofable header.
pub async fn relay_receive(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authenticate(&headers, &state.config().push_auth_token)?;

    let batch: RelayBatch = ciborium::from_reader(body.as_ref())
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid CBOR relay body"))?;

    accept_events(&state, batch.events, EventOrigin::Peer)?;
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
