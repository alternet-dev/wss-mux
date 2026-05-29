use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use futures_util::Stream;
use serde::Deserialize;
use serde_json::Value;

use crate::auth::{audience_admits, AuthError};
use crate::config::Config;
use crate::connection::{outbound_channel, ConnId, Outbound, OutboundRx};
use crate::dispatcher::{dispatch, EventOrigin};
use crate::envelope::{EnvelopePaths, EventEnvelope, ServerFrame};
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

/// `GET /events/:stream` — read events from a single stream as a
/// `text/event-stream` response. Sibling of `POST /events` (produce):
/// path-based stream selection so the URL reads cleanly. Optional
/// `?key=<value>` narrows to a single key, matching `subscribe`
/// semantics.
///
/// Auth supports two paths, configurable per deployment:
///
/// 1. **Shared bearer** — if `WSS_MUX_READ_AUTH_TOKEN` is set,
///    `Authorization: Bearer <that-token>` is admitted without any
///    audience check (full read across streams). Designed for
///    service-to-service consumers like a future `presence_svc`.
/// 2. **JWT** — the connecting token's principals must intersect the
///    stream's `subscribe` audience. Reuses the same validators as
///    WS subscribe (handshake key or OIDC).
///
/// Try shared-bearer first (cheap constant-time compare); fall back to
/// JWT. Both unset ⇒ 401 on every request.
///
/// No replay support: a reconnecting consumer picks up new events from
/// the reconnect point forward. `Last-Event-ID` is not honored.
///
/// Backpressure: per-consumer queue depth uses the same
/// `WSS_MUX_QUEUE_DEPTH` knob as WS subscribers. On overflow, the SSE
/// stream emits a final `event: error\ndata: overflow\n\n` and ends.
pub async fn sse_read(
    Path(stream_name): Path<String>,
    Query(query): Query<SseQuery>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Sse<SseSubscription>, (StatusCode, &'static str)> {
    let bearer =
        bearer_token(&headers).ok_or((StatusCode::UNAUTHORIZED, "missing or malformed bearer"))?;
    let auth = authenticate_read(&state, bearer)?;

    let manifest = state
        .manifest()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "manifest not loaded"))?;
    let stream_cfg = manifest
        .stream(&stream_name)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "unknown stream"))?;

    // The JWT path is audience-checked; the shared-bearer path is not
    // (the shared bearer carries no principal context — it admits full
    // read across streams, matching the produce-side `PUSH_AUTH_TOKEN`).
    if let ReadAuth::Jwt { principals } = &auth {
        if !audience_admits(principals, &stream_cfg.subscribe) {
            return Err((
                StatusCode::FORBIDDEN,
                "principals do not intersect stream subscribe audience",
            ));
        }
    }

    // Register as a one-shot subscription. Sub id is a constant — there
    // is one subscription per SSE connection (the path selects the
    // stream; the optional query parameter narrows by key).
    let conn_id = state.next_conn_id();
    let sub_id = "sse".to_string();
    let cap = stream_cfg.queue_depth.unwrap_or(state.config().queue_depth);
    let (sub_tx, sub_rx) = outbound_channel(cap);
    // Control is unbounded; it carries the keep-open `overflow` error
    // from the dispatcher, which is what triggers the final SSE error
    // event before this stream ends.
    let (ctl_tx, ctl_rx) = outbound_channel(0);

    state.register_control(conn_id, ctl_tx);
    state.register_sub_sender(conn_id, &sub_id, sub_tx, cap);
    state
        .registry()
        .subscribe(&stream_name, conn_id, sub_id.clone(), query.key.clone());
    state.metrics().subscriptions_active.inc();

    let sub = SseSubscription {
        state: state.clone(),
        conn_id,
        stream_name,
        sub_id,
        sub_rx,
        ctl_rx,
        closing: false,
    };

    // A periodic SSE comment keeps idle connections from being torn
    // down by intermediaries (proxies/LBs) that drop quiet HTTP/1.1
    // streams. Default cadence (15s) is fine.
    Ok(Sse::new(sub).keep_alive(KeepAlive::default()))
}

/// Query string for [`sse_read`]. `key` mirrors the `subscribe` frame's
/// optional key narrowing.
#[derive(Debug, Deserialize)]
pub struct SseQuery {
    #[serde(default)]
    pub key: Option<String>,
}

/// Which auth path admitted the SSE request — relevant because the
/// JWT path is audience-checked while the shared-bearer path is not.
enum ReadAuth {
    SharedBearer,
    Jwt { principals: Vec<String> },
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

fn authenticate_read(
    state: &AppState,
    bearer: &str,
) -> Result<ReadAuth, (StatusCode, &'static str)> {
    if let Some(token) = state.config().read_auth_token.as_deref() {
        if subtle_eq(bearer.as_bytes(), token.as_bytes()) {
            return Ok(ReadAuth::SharedBearer);
        }
    }
    let auth_result = match (state.handshake_verifier(), state.oidc_verifier()) {
        (Some(v), _) => v.validate(bearer),
        (_, Some(o)) => o.validate(bearer, &state.jwks()),
        (None, None) => return Err((StatusCode::UNAUTHORIZED, "no validator configured")),
    };
    match auth_result {
        Ok(claims) => Ok(ReadAuth::Jwt {
            principals: claims.principals,
        }),
        Err(AuthError::Expired) => Err((StatusCode::UNAUTHORIZED, "token expired")),
        Err(AuthError::Invalid(_)) => Err((StatusCode::UNAUTHORIZED, "invalid token")),
    }
}

/// SSE response body. Holds the per-sub receiver (where the dispatcher
/// writes event frames) and the control receiver (where overflow shows
/// up). `Drop` is the cleanup hook: when the client disconnects axum
/// drops the response body, which drops this, which unregisters the
/// subscription so the dispatcher stops trying to write to it.
pub struct SseSubscription {
    state: AppState,
    conn_id: ConnId,
    stream_name: String,
    sub_id: String,
    sub_rx: OutboundRx,
    ctl_rx: OutboundRx,
    /// Set after emitting the final overflow event so the next poll
    /// returns `None` and the response stream ends.
    closing: bool,
}

impl Stream for SseSubscription {
    type Item = Result<Event, Infallible>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();
        if me.closing {
            return Poll::Ready(None);
        }

        // Control gets priority: an overflow on the control channel
        // means this subscription has been dropped by the dispatcher;
        // we want the SSE consumer to see *why* before the stream ends.
        match Pin::new(&mut me.ctl_rx).poll_next(cx) {
            Poll::Ready(Some(Outbound::Frame(ServerFrame::Error { code, .. })))
                if code == "overflow" =>
            {
                me.closing = true;
                return Poll::Ready(Some(Ok(Event::default().event("error").data("overflow"))));
            }
            // Any other control item is unexpected for SSE — fall
            // through to the event loop and try again on the next poll.
            Poll::Ready(Some(_)) => {}
            Poll::Ready(None) => {}
            Poll::Pending => {}
        }

        match Pin::new(&mut me.sub_rx).poll_next(cx) {
            Poll::Ready(Some(Outbound::Frame(mut frame))) => {
                // The dispatcher echoes the sub id in every event
                // frame; SSE consumers don't supply or need it (one
                // stream-key tuple per response), so blank it.
                if let ServerFrame::Event { id, .. } = &mut frame {
                    id.clear();
                }
                let json = serde_json::to_string(&frame).unwrap_or_default();
                Poll::Ready(Some(Ok(Event::default().data(json))))
            }
            Poll::Ready(Some(_)) => Poll::Pending,
            Poll::Ready(None) => {
                me.closing = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for SseSubscription {
    fn drop(&mut self) {
        self.state
            .registry()
            .unsubscribe(&self.stream_name, self.conn_id, &self.sub_id);
        self.state.remove_connection(self.conn_id);
        self.state.metrics().subscriptions_active.dec();
    }
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
