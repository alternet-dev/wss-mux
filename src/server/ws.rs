use std::borrow::Cow;
use std::collections::HashMap;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{mpsc, watch};

use crate::auth::{audience_admits, validate_token, AuthError};
use crate::connection::{ConnId, Outbound};
use crate::envelope::{ClientFrame, ServerFrame};
use crate::error::ProtocolError;
use crate::server::AppState;

pub const SUBPROTOCOL: &str = "wss-mux.v1";

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    if !client_offers_subprotocol(&headers) {
        return (StatusCode::BAD_REQUEST, "subprotocol required: wss-mux.v1").into_response();
    }
    ws.protocols([SUBPROTOCOL])
        .on_upgrade(move |socket| handle_socket(socket, state))
}

fn client_offers_subprotocol(headers: &HeaderMap) -> bool {
    headers
        .get(header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').any(|p| p.trim() == SUBPROTOCOL))
        .unwrap_or(false)
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let conn_id = state.next_conn_id();
    let (data_tx, data_rx) = mpsc::channel::<Outbound>(state.config().queue_depth);
    let (abort_tx, abort_rx) = watch::channel(false);
    state.register_connection(conn_id, data_tx.clone(), abort_tx);

    state.metrics().connections_total.inc();
    state.metrics().connections_active.inc();

    let (writer, reader) = socket.split();
    let writer_task = tokio::spawn(writer_loop(writer, data_rx, abort_rx.clone()));

    reader_loop(reader, conn_id, &state, data_tx, abort_rx).await;

    state.unregister_connection(conn_id);
    state.metrics().connections_active.dec();
    let _ = writer_task.await;
}

async fn writer_loop(
    mut writer: SplitSink<WebSocket, Message>,
    mut rx: mpsc::Receiver<Outbound>,
    mut abort_rx: watch::Receiver<bool>,
) {
    // Mark the initial `false` as seen so changed() only fires on a real
    // transition. Without this, changed() returns Ready immediately the
    // first time it's polled.
    abort_rx.borrow_and_update();
    // Disable the abort branch once it can no longer signal — either the
    // sender dropped or we observed a non-true value. Otherwise the biased
    // select would keep starving the data channel.
    let mut abort_active = true;
    loop {
        tokio::select! {
            biased;
            res = abort_rx.changed(), if abort_active => {
                if res.is_ok() && *abort_rx.borrow() {
                    emit_overflow_close(&mut writer).await;
                    return;
                }
                abort_active = false;
            }
            msg = rx.recv() => {
                match msg {
                    Some(Outbound::Frame(frame)) => {
                        let Ok(json) = serde_json::to_string(&frame) else {
                            continue;
                        };
                        if writer.send(Message::Text(json)).await.is_err() {
                            return;
                        }
                    }
                    Some(Outbound::Close { code, reason, frame }) => {
                        if let Some(frame) = frame {
                            if let Ok(json) = serde_json::to_string(&frame) {
                                let _ = writer.send(Message::Text(json)).await;
                            }
                        }
                        let close = CloseFrame { code, reason: Cow::Owned(reason) };
                        let _ = writer.send(Message::Close(Some(close))).await;
                        return;
                    }
                    None => break,
                }
            }
        }
    }
    let _ = writer.send(Message::Close(None)).await;
}

async fn emit_overflow_close(writer: &mut SplitSink<WebSocket, Message>) {
    let err = ServerFrame::Error {
        code: "overflow".into(),
        message: "per-connection send queue overflowed".into(),
        id: None,
    };
    if let Ok(json) = serde_json::to_string(&err) {
        let _ = writer.send(Message::Text(json)).await;
    }
    let close = CloseFrame {
        code: 4429,
        reason: Cow::Borrowed("overflow"),
    };
    let _ = writer.send(Message::Close(Some(close))).await;
}

fn parse_client_frame(text: &str) -> Result<ClientFrame, ProtocolError> {
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

async fn reader_loop(
    mut reader: SplitStream<WebSocket>,
    conn_id: ConnId,
    state: &AppState,
    tx: mpsc::Sender<Outbound>,
    mut abort_rx: watch::Receiver<bool>,
) {
    let mut principals: Option<Vec<String>> = None;
    let mut subs: HashMap<String, (String, Option<String>)> = HashMap::new();
    let mut explicit_close = false;
    let mut aborted = false;

    abort_rx.borrow_and_update();
    let mut abort_active = true;
    loop {
        let msg = tokio::select! {
            biased;
            res = abort_rx.changed(), if abort_active => {
                if res.is_ok() && *abort_rx.borrow() {
                    aborted = true;
                    break;
                }
                abort_active = false;
                continue;
            }
            msg = reader.next() => match msg {
                Some(m) => m,
                None => break,
            }
        };

        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Binary(_)) => {
                let _ = tx.try_send(ProtocolError::BadFrame.to_outbound());
                explicit_close = true;
                break;
            }
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };

        let frame = match parse_client_frame(text.as_str()) {
            Ok(f) => f,
            Err(err) => {
                let _ = tx.try_send(err.to_outbound());
                explicit_close = true;
                break;
            }
        };

        match frame {
            ClientFrame::Auth { token } => {
                if principals.is_some() {
                    continue;
                }
                match validate_token(&token, &state.config().handshake_signing_key) {
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
                    .and_then(|m| m.stream(&stream))
                    .map(|s| s.audience.clone())
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

    // If we exited via the abort path, the writer is already taking care of
    // the overflow error + close 4429; don't queue a competing normal_close.
    if !explicit_close && !aborted {
        let _ = tx.try_send(Outbound::normal_close());
    }
}
