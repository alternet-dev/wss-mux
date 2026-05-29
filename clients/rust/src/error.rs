//! Typed errors surfaced by the SDK.

use crate::types::ErrorCode;
use thiserror::Error;

/// Top-level error type for the SDK.
#[derive(Debug, Error)]
pub enum WssMuxError {
    /// The server sent an `error` frame. Includes the documented
    /// code/message and, when applicable, the subscription or publish
    /// `id` the error pertains to.
    #[error("server error: {code:?} {message}")]
    Protocol {
        code: ErrorCode,
        message: String,
        id: Option<String>,
    },

    /// The WebSocket connection closed unexpectedly. Holds the close
    /// code (per `docs/protocol.md` and RFC 6455) and the optional
    /// reason text the server sent.
    #[error("connection closed: code={code} reason={reason:?}")]
    ConnectionClosed { code: u16, reason: String },

    /// Reconnect budget exhausted.
    #[error("reconnect attempts exhausted")]
    ReconnectExhausted,

    /// The SDK is closed; no further operations are accepted.
    #[error("client is closed")]
    Closed,

    /// Invalid use of the SDK (bad builder args, missing callback,
    /// etc.).
    #[error("client usage error: {0}")]
    Usage(String),

    /// Network or transport-level failure during connect or send.
    #[error("transport error: {0}")]
    Transport(String),
}

impl WssMuxError {
    /// Returns the protocol error code if this is a Protocol variant.
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::Protocol { code, .. } => Some(*code),
            _ => None,
        }
    }
}
