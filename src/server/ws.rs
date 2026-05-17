use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::watch;

use crate::auth::{audience_admits, AuthError};
use crate::connection::{outbound_channel, ConnId, Outbound, OutboundRx, OutboundTx};
use crate::envelope::{ClientFrame, ServerFrame};
use crate::error::ProtocolError;
use crate::manifest::Manifest;
use crate::ratelimit::TokenBucket;
use crate::server::metrics::{RevokeReason, RevokeReasonLabel};
use crate::server::AppState;

pub const SUBPROTOCOL: &str = "wss-mux.v1";
pub const SUBPROTOCOL_CBOR: &str = "wss-mux.v1.cbor";

/// Per-connection wire encoding, fixed at subprotocol negotiation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    /// `wss-mux.v1` — frames as UTF-8 JSON text messages.
    Json,
    /// `wss-mux.v1.cbor` — frames as CBOR binary messages.
    Cbor,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    let Some(codec) = negotiate_codec(&headers) else {
        return (
            StatusCode::BAD_REQUEST,
            "subprotocol required: wss-mux.v1 or wss-mux.v1.cbor",
        )
            .into_response();
    };
    // Server preference order: CBOR first, so a client offering both
    // gets CBOR — matching negotiate_codec.
    ws.protocols([SUBPROTOCOL_CBOR, SUBPROTOCOL])
        .on_upgrade(move |socket| handle_socket(socket, state, codec))
}

/// Pick the codec from the client's `Sec-WebSocket-Protocol` offer. CBOR
/// wins if offered anywhere in the list; otherwise plain `wss-mux.v1`
/// selects JSON. `None` means no compatible subprotocol was offered.
fn negotiate_codec(headers: &HeaderMap) -> Option<Codec> {
    let offered = headers
        .get(header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|v| v.to_str().ok())?;
    let mut has_json = false;
    for proto in offered.split(',').map(str::trim) {
        if proto == SUBPROTOCOL_CBOR {
            return Some(Codec::Cbor);
        }
        if proto == SUBPROTOCOL {
            has_json = true;
        }
    }
    has_json.then_some(Codec::Json)
}

/// Encode a server frame for the connection's codec.
fn encode_frame(codec: Codec, frame: &ServerFrame) -> Option<Message> {
    match codec {
        Codec::Json => serde_json::to_string(frame).ok().map(Message::Text),
        Codec::Cbor => {
            let mut buf = Vec::new();
            ciborium::into_writer(frame, &mut buf).ok()?;
            Some(Message::Binary(buf))
        }
    }
}

async fn handle_socket(socket: WebSocket, state: AppState, codec: Codec) {
    let conn_id = state.next_conn_id();
    let (data_tx, data_rx) = outbound_channel(state.config().queue_depth);
    state.register_connection(conn_id, data_tx.clone());

    state.metrics().connections_total.inc();
    state.metrics().connections_active.inc();

    let manifest_rx = state.subscribe_manifest();

    let (writer, reader) = socket.split();
    let writer_task = tokio::spawn(writer_loop(writer, data_rx, codec, conn_id, state.clone()));

    reader_loop(reader, conn_id, &state, data_tx, manifest_rx, codec).await;

    state.unregister_connection(conn_id);
    state.metrics().connections_active.dec();
    let _ = writer_task.await;
}

async fn writer_loop(
    mut writer: SplitSink<WebSocket, Message>,
    mut rx: OutboundRx,
    codec: Codec,
    conn_id: ConnId,
    state: AppState,
) {
    while let Some(msg) = rx.recv().await {
        match msg {
            Outbound::Frame(frame) => {
                // An Event frame consumed a per-subscription slot the
                // dispatcher reserved before enqueuing it.
                let sub_id = match &frame {
                    ServerFrame::Event { id, .. } => Some(id.clone()),
                    _ => None,
                };
                if let Some(msg) = encode_frame(codec, &frame) {
                    if writer.send(msg).await.is_err() {
                        return;
                    }
                }
                // The frame has left the shared channel; free the
                // reserved slot so the subscription can receive more
                // (release on an absent count is a no-op).
                if let Some(id) = sub_id {
                    state.sub_queues().release(conn_id, &id);
                }
            }
            Outbound::Close {
                code,
                reason,
                frame,
            } => {
                if let Some(frame) = frame {
                    if let Some(msg) = encode_frame(codec, &frame) {
                        let _ = writer.send(msg).await;
                    }
                }
                let close = CloseFrame {
                    code,
                    reason: Cow::Owned(reason),
                };
                let _ = writer.send(Message::Close(Some(close))).await;
                return;
            }
        }
    }
    let _ = writer.send(Message::Close(None)).await;
}

/// Two-stage JSON parse: any read failure is `bad_frame`; a parsed body
/// with an unrecognized `type` is `unknown_frame_type`.
fn parse_client_frame_json(text: &str) -> Result<ClientFrame, ProtocolError> {
    let value: Value = serde_json::from_str(text).map_err(|_| ProtocolError::BadFrame)?;
    let type_str = value
        .get("type")
        .and_then(Value::as_str)
        .ok_or(ProtocolError::BadFrame)?;
    match type_str {
        "auth" | "subscribe" | "unsubscribe" => {
            serde_json::from_value(value).map_err(|_| ProtocolError::BadFrame)
        }
        _ => Err(ProtocolError::UnknownFrameType),
    }
}

/// CBOR analogue of `parse_client_frame_json` — same two-stage logic so
/// `bad_frame` and `unknown_frame_type` stay distinguishable on binary
/// connections.
fn parse_client_frame_cbor(bytes: &[u8]) -> Result<ClientFrame, ProtocolError> {
    let value: ciborium::value::Value =
        ciborium::from_reader(bytes).map_err(|_| ProtocolError::BadFrame)?;
    let type_str = value
        .as_map()
        .and_then(|entries| {
            entries
                .iter()
                .find_map(|(k, v)| (k.as_text() == Some("type")).then(|| v.as_text()).flatten())
        })
        .ok_or(ProtocolError::BadFrame)?;
    match type_str {
        "auth" | "subscribe" | "unsubscribe" => {
            value.deserialized().map_err(|_| ProtocolError::BadFrame)
        }
        _ => Err(ProtocolError::UnknownFrameType),
    }
}

/// The correlation id to echo in an `error` frame for a given client
/// frame — `subscribe`/`unsubscribe` carry one, `auth` does not.
fn client_frame_id(frame: &ClientFrame) -> Option<String> {
    match frame {
        ClientFrame::Auth { .. } => None,
        ClientFrame::Subscribe { id, .. } | ClientFrame::Unsubscribe { id } => Some(id.clone()),
    }
}

/// On a hot-reload, decide whether an existing subscription survives the
/// new manifest snapshot. `None` means it's still valid; `Some(reason)`
/// distinguishes a removed stream from an audience that no longer
/// intersects the connection's principals.
fn revoke_reason(
    manifest: Option<&Manifest>,
    principals: &[String],
    stream: &str,
) -> Option<RevokeReason> {
    match manifest.and_then(|m| m.stream(stream)) {
        None => Some(RevokeReason::UnknownStream),
        Some(s) if !audience_admits(principals, &s.audience) => Some(RevokeReason::Unauthorized),
        Some(_) => None,
    }
}

async fn reader_loop(
    mut reader: SplitStream<WebSocket>,
    conn_id: ConnId,
    state: &AppState,
    tx: OutboundTx,
    mut manifest_rx: watch::Receiver<Option<Arc<Manifest>>>,
    codec: Codec,
) {
    let mut principals: Option<Vec<String>> = None;
    let mut subs: HashMap<String, (String, Option<String>)> = HashMap::new();
    let mut explicit_close = false;

    // Per-connection inbound limiter. `rate == 0` disables it entirely.
    let cfg = state.config();
    let mut rate_limiter = (cfg.inbound_rate_per_sec > 0)
        .then(|| TokenBucket::new(cfg.inbound_burst, cfg.inbound_rate_per_sec, Instant::now()));

    // Mark the current manifest as seen so changed() only fires on a real
    // SIGHUP swap, not the value already in place when this task starts.
    manifest_rx.borrow_and_update();
    let mut manifest_active = true;
    loop {
        let msg = tokio::select! {
            biased;
            res = manifest_rx.changed(), if manifest_active => {
                if res.is_err() {
                    manifest_active = false;
                    continue;
                }
                let snapshot = manifest_rx.borrow_and_update().clone();
                if let Some(p) = principals.as_ref() {
                    let revoked: Vec<(String, String, RevokeReason)> = subs
                        .iter()
                        .filter_map(|(sub_id, (stream, _))| {
                            revoke_reason(snapshot.as_deref(), p, stream)
                                .map(|reason| (sub_id.clone(), stream.clone(), reason))
                        })
                        .collect();
                    for (sub_id, stream, reason) in revoked {
                        let err = match &reason {
                            RevokeReason::UnknownStream => {
                                ProtocolError::UnknownStream { id: sub_id.clone() }
                            }
                            RevokeReason::Unauthorized => {
                                ProtocolError::UnauthorizedSubscribe { id: sub_id.clone() }
                            }
                        };
                        let _ = tx.try_send(err.to_outbound());
                        state.registry().unsubscribe(&stream, conn_id, &sub_id);
                        state.metrics().subscriptions_active.dec();
                        state
                            .metrics()
                            .subscriptions_revoked
                            .get_or_create(&RevokeReasonLabel { reason })
                            .inc();
                        subs.remove(&sub_id);
                    }
                }
                continue;
            }
            msg = reader.next() => match msg {
                Some(m) => m,
                None => break,
            }
        };

        // Decode per the negotiated codec. A wire-type mismatch (text on
        // a CBOR connection or binary on a JSON one) is `bad_frame`, the
        // same close-4400 path as a malformed body.
        let parsed = match msg {
            Ok(Message::Text(t)) if codec == Codec::Json => parse_client_frame_json(t.as_str()),
            Ok(Message::Binary(b)) if codec == Codec::Cbor => parse_client_frame_cbor(&b),
            Ok(Message::Text(_)) | Ok(Message::Binary(_)) => Err(ProtocolError::BadFrame),
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };

        let frame = match parsed {
            Ok(f) => f,
            Err(err) => {
                let _ = tx.try_send(err.to_outbound());
                explicit_close = true;
                break;
            }
        };

        // Rate-limit gate: a throttled frame is dropped (not processed),
        // the client gets a keep-open `rate_limited` error, and the
        // connection survives.
        if let Some(bucket) = rate_limiter.as_mut() {
            if !bucket.try_take(Instant::now()) {
                let _ = tx.try_send(
                    ProtocolError::RateLimited {
                        id: client_frame_id(&frame),
                    }
                    .to_outbound(),
                );
                state.metrics().frames_rate_limited.inc();
                continue;
            }
        }

        match frame {
            ClientFrame::Auth { token } => {
                if principals.is_some() {
                    continue;
                }
                match state.handshake_verifier().validate(&token) {
                    Ok(claims) => {
                        tracing::debug!(conn_id, sub = %claims.sub, "authenticated");
                        principals = Some(claims.principals);
                    }
                    Err(AuthError::Expired) => {
                        let _ = tx.try_send(ProtocolError::ExpiredToken.to_outbound());
                        explicit_close = true;
                        break;
                    }
                    Err(AuthError::Invalid(_)) => {
                        let _ =
                            tx.try_send(ProtocolError::Unauthenticated { id: None }.to_outbound());
                        explicit_close = true;
                        break;
                    }
                }
            }
            ClientFrame::Subscribe { id, stream, key } => {
                let Some(p) = principals.as_ref() else {
                    let _ = tx.try_send(
                        ProtocolError::Unauthenticated {
                            id: Some(id.clone()),
                        }
                        .to_outbound(),
                    );
                    explicit_close = true;
                    break;
                };
                let Some(audience) = state
                    .manifest()
                    .and_then(|m| m.stream(&stream).map(|s| s.audience.clone()))
                else {
                    let _ = tx.try_send(ProtocolError::UnknownStream { id }.to_outbound());
                    continue;
                };
                if !audience_admits(p, &audience) {
                    let _ = tx.try_send(ProtocolError::UnauthorizedSubscribe { id }.to_outbound());
                    continue;
                }
                if subs.contains_key(&id) {
                    let _ =
                        tx.try_send(ProtocolError::DuplicateSubscriptionId { id }.to_outbound());
                    continue;
                }
                state
                    .registry()
                    .subscribe(&stream, conn_id, id.clone(), key.clone());
                state.metrics().subscriptions_active.inc();
                subs.insert(id, (stream, key));
            }
            ClientFrame::Unsubscribe { id } => {
                if let Some((stream, _)) = subs.remove(&id) {
                    state.registry().unsubscribe(&stream, conn_id, &id);
                    state.metrics().subscriptions_active.dec();
                }
            }
        }
    }

    for (sub_id, (stream, _)) in &subs {
        state.registry().unsubscribe(stream, conn_id, sub_id);
        state.metrics().subscriptions_active.dec();
    }

    // A typed close (e.g. a 4400/4401 error) was already queued via
    // `explicit_close`; otherwise signal a normal close so the writer
    // exits on an explicit signal rather than channel-drop alone.
    if !explicit_close {
        let _ = tx.try_send(Outbound::normal_close());
    }
}
