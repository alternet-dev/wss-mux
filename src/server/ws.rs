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
use tokio::sync::{mpsc, watch};
use tokio_stream::StreamMap;

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

/// Effective per-subscription send-queue depth: the stream's manifest
/// `queue_depth` if set (`0` ⇒ unlimited), else the global default
/// (`WSS_MUX_QUEUE_DEPTH`, also `0` ⇒ unlimited). The value is passed
/// straight to `outbound_channel` (which treats `0` as unbounded).
fn effective_cap(manifest: Option<&Manifest>, stream: &str, global: usize) -> usize {
    manifest
        .and_then(|m| m.stream(stream).and_then(|s| s.queue_depth))
        .unwrap_or(global)
}

async fn handle_socket(socket: WebSocket, state: AppState, codec: Codec) {
    let conn_id = state.next_conn_id();
    // The control channel carries connection-level frames + typed
    // closes + keep-open overflow errors. Unbounded: low-volume and
    // must never be shed.
    let (control_tx, control_rx) = outbound_channel(0);
    state.register_control(conn_id, control_tx.clone());
    // Reader → writer: hands each new (or resized) per-sub receiver to
    // the writer's StreamMap.
    let (reg_tx, reg_rx) = mpsc::unbounded_channel::<(String, OutboundRx)>();

    state.metrics().connections_total.inc();
    state.metrics().connections_active.inc();

    let manifest_rx = state.subscribe_manifest();

    let (writer, reader) = socket.split();
    let writer_task = tokio::spawn(writer_loop(writer, control_rx, reg_rx, codec));

    reader_loop(
        reader,
        conn_id,
        &state,
        control_tx,
        reg_tx,
        manifest_rx,
        codec,
    )
    .await;

    // Drops the control sender and every per-sub sender for this
    // connection; the writer's control channel + StreamMap then all
    // close and it exits.
    state.remove_connection(conn_id);
    state.metrics().connections_active.dec();
    let _ = writer_task.await;
}

async fn writer_loop(
    mut writer: SplitSink<WebSocket, Message>,
    mut control_rx: OutboundRx,
    mut reg_rx: mpsc::UnboundedReceiver<(String, OutboundRx)>,
    codec: Codec,
) {
    // The dynamic set of per-subscription receivers, merged fairly.
    // A receiver ends when its sender is dropped (unsubscribe /
    // overflow / teardown / cap-recreate), and `StreamMap` evicts it.
    let mut subs: StreamMap<String, OutboundRx> = StreamMap::new();
    let mut reg_active = true;
    loop {
        tokio::select! {
            biased;
            // 1) Connection-level frames and typed closes — highest
            //    priority so a close/error always wins.
            ctl = control_rx.recv() => match ctl {
                Some(Outbound::Frame(frame)) => {
                    if let Some(m) = encode_frame(codec, &frame) {
                        if writer.send(m).await.is_err() {
                            return;
                        }
                    }
                }
                Some(Outbound::Close { code, reason, frame }) => {
                    if let Some(frame) = frame {
                        if let Some(m) = encode_frame(codec, &frame) {
                            let _ = writer.send(m).await;
                        }
                    }
                    let close = CloseFrame { code, reason: Cow::Owned(reason) };
                    let _ = writer.send(Message::Close(Some(close))).await;
                    return;
                }
                None => break, // control gone ⇒ connection torn down
            },
            // 2) (Re)register a per-subscription receiver. `insert`
            //    replacing an existing key drops the old receiver —
            //    exactly the SIGHUP cap-recreate path.
            reg = reg_rx.recv(), if reg_active => match reg {
                Some((sub_id, rx)) => { subs.insert(sub_id, rx); }
                None => { reg_active = false; }
            },
            // 3) Per-subscription event frames, fairly merged. Guarded
            //    so an empty StreamMap (which yields None immediately)
            //    doesn't busy-spin.
            merged = subs.next(), if !subs.is_empty() => {
                if let Some((_sub, Outbound::Frame(frame))) = merged {
                    if let Some(m) = encode_frame(codec, &frame) {
                        if writer.send(m).await.is_err() {
                            return;
                        }
                    }
                }
                // Per-sub channels only ever carry Frame(Event); a
                // None here just means the map drained to empty.
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
    control_tx: OutboundTx,
    reg_tx: mpsc::UnboundedSender<(String, OutboundRx)>,
    mut manifest_rx: watch::Receiver<Option<Arc<Manifest>>>,
    codec: Codec,
) {
    let mut principals: Option<Vec<String>> = None;
    // sub_id -> (stream, key, the effective cap its channel was built
    // with — tracked so a SIGHUP queue_depth change can recreate it).
    let mut subs: HashMap<String, (String, Option<String>, usize)> = HashMap::new();
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
                    // (a) Revoke subscriptions the new manifest no
                    //     longer admits (unknown stream / lost audience).
                    let revoked: Vec<(String, String, RevokeReason)> = subs
                        .iter()
                        .filter_map(|(sub_id, (stream, _, _))| {
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
                        let _ = control_tx.try_send(err.to_outbound());
                        state.remove_sub_sender(conn_id, &sub_id);
                        state.registry().unsubscribe(&stream, conn_id, &sub_id);
                        state.metrics().subscriptions_active.dec();
                        state
                            .metrics()
                            .subscriptions_revoked
                            .get_or_create(&RevokeReasonLabel { reason })
                            .inc();
                        subs.remove(&sub_id);
                    }
                    // (b) Survivors whose effective queue_depth changed:
                    //     recreate the channel at the new size and
                    //     re-register. The writer's StreamMap.insert
                    //     drops the old receiver; any frames buffered in
                    //     the old channel are dropped (acceptable per
                    //     the at-most-once contract; SIGHUP is rare).
                    let global = state.config().queue_depth;
                    let resized: Vec<(String, usize)> = subs
                        .iter()
                        .filter_map(|(sub_id, (stream, _, cur_cap))| {
                            let new_cap = effective_cap(snapshot.as_deref(), stream, global);
                            (new_cap != *cur_cap).then(|| (sub_id.clone(), new_cap))
                        })
                        .collect();
                    for (sub_id, new_cap) in resized {
                        let (sub_tx, sub_rx) = outbound_channel(new_cap);
                        state.register_sub_sender(conn_id, &sub_id, sub_tx, new_cap);
                        let _ = reg_tx.send((sub_id.clone(), sub_rx));
                        if let Some(entry) = subs.get_mut(&sub_id) {
                            entry.2 = new_cap;
                        }
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
                let _ = control_tx.try_send(err.to_outbound());
                explicit_close = true;
                break;
            }
        };

        // Rate-limit gate: a throttled frame is dropped (not processed),
        // the client gets a keep-open `rate_limited` error, and the
        // connection survives.
        if let Some(bucket) = rate_limiter.as_mut() {
            if !bucket.try_take(Instant::now()) {
                let _ = control_tx.try_send(
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
                        let _ = control_tx.try_send(ProtocolError::ExpiredToken.to_outbound());
                        explicit_close = true;
                        break;
                    }
                    Err(AuthError::Invalid(_)) => {
                        let _ = control_tx
                            .try_send(ProtocolError::Unauthenticated { id: None }.to_outbound());
                        explicit_close = true;
                        break;
                    }
                }
            }
            ClientFrame::Subscribe { id, stream, key } => {
                let Some(p) = principals.as_ref() else {
                    let _ = control_tx.try_send(
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
                    let _ = control_tx.try_send(ProtocolError::UnknownStream { id }.to_outbound());
                    continue;
                };
                if !audience_admits(p, &audience) {
                    let _ = control_tx
                        .try_send(ProtocolError::UnauthorizedSubscribe { id }.to_outbound());
                    continue;
                }
                if subs.contains_key(&id) {
                    let _ = control_tx
                        .try_send(ProtocolError::DuplicateSubscriptionId { id }.to_outbound());
                    continue;
                }
                // (c): one channel per subscription, sized from the
                // stream's effective queue_depth at subscribe time. The
                // receiver is handed to the writer's StreamMap.
                let cap = effective_cap(
                    state.manifest().as_deref(),
                    &stream,
                    state.config().queue_depth,
                );
                let (sub_tx, sub_rx) = outbound_channel(cap);
                state.register_sub_sender(conn_id, &id, sub_tx, cap);
                let _ = reg_tx.send((id.clone(), sub_rx));
                state
                    .registry()
                    .subscribe(&stream, conn_id, id.clone(), key.clone());
                state.metrics().subscriptions_active.inc();
                subs.insert(id, (stream, key, cap));
            }
            ClientFrame::Unsubscribe { id } => {
                if let Some((stream, _, _)) = subs.remove(&id) {
                    state.remove_sub_sender(conn_id, &id);
                    state.registry().unsubscribe(&stream, conn_id, &id);
                    state.metrics().subscriptions_active.dec();
                }
            }
        }
    }

    // Per-sub senders are dropped by `remove_connection` in
    // `handle_socket`; here we just clear registry bindings + the gauge.
    for (sub_id, (stream, _, _)) in &subs {
        state.registry().unsubscribe(stream, conn_id, sub_id);
        state.metrics().subscriptions_active.dec();
    }

    // A typed close (e.g. a 4400/4401 error) was already queued via
    // `explicit_close`; otherwise signal a normal close so the writer
    // exits on an explicit signal rather than channel-drop alone.
    if !explicit_close {
        let _ = control_tx.try_send(Outbound::normal_close());
    }
}
