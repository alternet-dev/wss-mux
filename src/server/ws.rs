use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::auth::{audience_admits, validate_token};
use crate::connection::ConnId;
use crate::envelope::{ClientFrame, ServerFrame};
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
    let (tx, rx) = mpsc::channel::<ServerFrame>(state.config().queue_depth);
    state.register_connection(conn_id, tx.clone());

    let (writer, reader) = socket.split();
    let writer_task = tokio::spawn(writer_loop(writer, rx));
    reader_loop(reader, conn_id, state.clone()).await;

    state.unregister_connection(conn_id);
    drop(tx);
    let _ = writer_task.await;
}

async fn writer_loop(
    mut writer: futures_util::stream::SplitSink<WebSocket, Message>,
    mut rx: mpsc::Receiver<ServerFrame>,
) {
    while let Some(frame) = rx.recv().await {
        let Ok(json) = serde_json::to_string(&frame) else {
            continue;
        };
        if writer.send(Message::Text(json)).await.is_err() {
            break;
        }
    }
    let _ = writer.close().await;
}

async fn reader_loop(
    mut reader: futures_util::stream::SplitStream<WebSocket>,
    conn_id: ConnId,
    state: AppState,
) {
    let mut principals: Option<Vec<String>> = None;
    let mut subs: HashMap<String, (String, Option<String>)> = HashMap::new();

    while let Some(msg) = reader.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };
        let Ok(frame) = serde_json::from_str::<ClientFrame>(text.as_str()) else {
            break;
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
                    Err(err) => {
                        tracing::debug!(conn_id, ?err, "auth failed; closing");
                        break;
                    }
                }
            }
            ClientFrame::Subscribe { id, stream, key } => {
                let Some(p) = principals.as_ref() else {
                    break;
                };
                let Some(audience) = state
                    .manifest()
                    .and_then(|m| m.stream(&stream))
                    .map(|s| s.audience.clone())
                else {
                    continue;
                };
                if !audience_admits(p, &audience) {
                    continue;
                }
                if subs.contains_key(&id) {
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
            }
        }
    }

    for (sub_id, (stream, _)) in &subs {
        state.registry().unsubscribe(stream, conn_id, sub_id);
    }
}
