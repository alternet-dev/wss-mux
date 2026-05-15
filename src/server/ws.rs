use std::borrow::Cow;
use std::collections::HashMap;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::auth::{audience_admits, validate_token, AuthError};
use crate::connection::{ConnId, Outbound};
use crate::envelope::ClientFrame;
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
    let (tx, rx) = mpsc::channel::<Outbound>(state.config().queue_depth);
    state.register_connection(conn_id, tx.clone());

    let (writer, reader) = socket.split();
    let writer_task = tokio::spawn(writer_loop(writer, rx));

    reader_loop(reader, conn_id, &state, tx).await;

    state.unregister_connection(conn_id);
    let _ = writer_task.await;
}

async fn writer_loop(mut writer: SplitSink<WebSocket, Message>, mut rx: mpsc::Receiver<Outbound>) {
    while let Some(out) = rx.recv().await {
        match out {
            Outbound::Frame(frame) => {
                let Ok(json) = serde_json::to_string(&frame) else {
                    continue;
                };
                if writer.send(Message::Text(json)).await.is_err() {
                    return;
                }
            }
            Outbound::Close {
                code,
                reason,
                frame,
            } => {
                if let Some(frame) = frame {
                    if let Ok(json) = serde_json::to_string(&frame) {
                        let _ = writer.send(Message::Text(json)).await;
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

/// Two-stage parse: any failure to read the JSON body is `bad_frame`; a
/// successfully parsed body with an unrecognized `type` field is
/// `unknown_frame_type`. Keeping these distinguishable is what
/// `docs/protocol.md` requires.
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
) {
    let mut principals: Option<Vec<String>> = None;
    let mut subs: HashMap<String, (String, Option<String>)> = HashMap::new();

    let mut explicit_close = false;

    while let Some(msg) = reader.next().await {
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
                subs.insert(id, (stream, key));
            }
            ClientFrame::Unsubscribe { id } => {
                if let Some((stream, _)) = subs.remove(&id) {
                    state.registry().unsubscribe(&stream, conn_id, &id);
                }
                // Idempotent — no error for unknown ids (per docs/protocol.md).
            }
        }
    }

    for (sub_id, (stream, _)) in &subs {
        state.registry().unsubscribe(stream, conn_id, sub_id);
    }

    if !explicit_close {
        let _ = tx.try_send(Outbound::normal_close());
    }
}
