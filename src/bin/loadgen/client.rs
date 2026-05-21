//! Black-box WebSocket client for the harness. A `[[bin]]` cannot share
//! `tests/integration/common.rs`, so the small helpers it needs are
//! reconstructed here on `tokio-tungstenite` directly.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use wss_mux::envelope::{ClientFrame, ServerFrame};

pub type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Open a WebSocket to `<ws_base>/v1/stream`, negotiating the JSON
/// `wss-mux.v1` subprotocol — the server rejects a connection that
/// offers none.
pub async fn connect(ws_base: &str) -> Result<Ws> {
    let url = format!("{}/v1/stream", ws_base.trim_end_matches('/'));
    let mut request = url
        .into_client_request()
        .context("build WebSocket request")?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        "wss-mux.v1"
            .parse()
            .expect("static subprotocol header is valid"),
    );
    let (stream, _response) = tokio_tungstenite::connect_async(request)
        .await
        .context("WebSocket connect")?;
    Ok(stream)
}

/// Serialize and send a client frame as a JSON text message.
pub async fn send_frame(ws: &mut Ws, frame: &ClientFrame) -> Result<()> {
    let text = serde_json::to_string(frame).context("serialize client frame")?;
    ws.send(WsMessage::Text(text))
        .await
        .context("WebSocket send")?;
    Ok(())
}

/// Authenticate, then subscribe to `stream` under `sub_id`, optionally
/// narrowed to `key`. Neither frame is acknowledged by the server;
/// readiness is confirmed out of band by polling
/// `wss_mux_subscriptions_active`.
pub async fn auth_and_subscribe(
    ws: &mut Ws,
    token: &str,
    sub_id: &str,
    stream: &str,
    key: Option<&str>,
) -> Result<()> {
    send_frame(
        ws,
        &ClientFrame::Auth {
            token: token.to_string(),
        },
    )
    .await?;
    send_frame(
        ws,
        &ClientFrame::Subscribe {
            id: sub_id.to_string(),
            stream: stream.to_string(),
            key: key.map(str::to_string),
        },
    )
    .await?;
    Ok(())
}

/// Outcome of waiting for the next server frame.
pub enum Recv {
    /// An `event` frame was delivered.
    Event(ServerFrame),
    /// An `error` frame was delivered (e.g. failed auth, overflow).
    Error,
    /// The wait timed out with nothing delivered.
    Idle,
    /// The socket closed or the stream ended.
    Closed,
}

/// Wait up to `timeout` for the next server frame, transparently
/// skipping ping/pong (tungstenite answers pings itself on read).
pub async fn next(ws: &mut Ws, timeout: Duration) -> Result<Recv> {
    loop {
        let message = match tokio::time::timeout(timeout, ws.next()).await {
            Err(_elapsed) => return Ok(Recv::Idle),
            Ok(None) => return Ok(Recv::Closed),
            Ok(Some(result)) => result.context("WebSocket receive")?,
        };
        return Ok(match message {
            WsMessage::Text(text) => {
                match serde_json::from_str::<ServerFrame>(&text).context("parse server frame")? {
                    frame @ ServerFrame::Event { .. } => Recv::Event(frame),
                    ServerFrame::Error { .. } => Recv::Error,
                }
            }
            WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Frame(_) => continue,
            WsMessage::Close(_) => Recv::Closed,
            WsMessage::Binary(_) => {
                return Err(anyhow!("unexpected binary frame on a JSON connection"))
            }
        });
    }
}
