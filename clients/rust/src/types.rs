//! Wire-protocol types for wss-mux.
//!
//! Authoritative source: `docs/protocol.md` in the wss-mux repo. These
//! types are duplicated from the server's `wss-mux::envelope` so the
//! client crate stays independent of the server's heavy dependency
//! graph (axum, prometheus, etc.).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Identifier the client picks for a subscription. Echoed by the
/// server on every event matching that subscription and on errors
/// pertaining to it.
pub type SubscriptionId = String;

/// Identifier the client picks for a publish. Echoed only on error
/// frames matching the publish.
pub type PublishId = String;

/// Frames the client sends to the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientFrame {
    /// First frame on every connection; bearer token.
    Auth { token: String },

    /// Bind a subscription within this connection.
    Subscribe {
        id: SubscriptionId,
        stream: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
    },

    /// Remove a subscription. Idempotent.
    Unsubscribe { id: SubscriptionId },

    /// Emit an event on a stream. Identical downstream semantics to
    /// HTTP `POST /events`; the server's per-stream `publish` audience
    /// gates which connections may publish to which streams.
    Publish {
        id: PublishId,
        stream: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        payload: Value,
    },
}

/// Frames the server sends to the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerFrame {
    /// An event matched a subscription.
    Event {
        id: SubscriptionId,
        stream: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        payload: Value,
    },

    /// The server reports a problem. Some codes are keep-open
    /// (the connection stays up); others mean the server is about
    /// to close.
    Error {
        code: ErrorCode,
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
}

/// Wire-level error codes. The full taxonomy lives in `docs/protocol.md`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UnknownFrameType,
    BadFrame,
    Unauthenticated,
    ExpiredToken,
    UnknownStream,
    UnauthorizedSubscribe,
    UnauthorizedPublish,
    PublishPayloadTooLarge,
    DuplicateSubscriptionId,
    RateLimited,
    Overflow,
}

/// WebSocket close codes used by wss-mux. `1000` is the normal close
/// from RFC 6455; `4400` and `4401` are application-defined per
/// `docs/protocol.md`.
pub const CLOSE_NORMAL: u16 = 1000;
pub const CLOSE_BAD_FRAME: u16 = 4400;
pub const CLOSE_UNAUTHENTICATED: u16 = 4401;

/// The subprotocol the SDK negotiates on upgrade. Internal — consumers
/// should not need to reference it.
pub(crate) const SUBPROTOCOL_JSON: &str = "wss-mux";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn subscribe_frame_roundtrip_with_key() {
        let f = ClientFrame::Subscribe {
            id: "sub-1".into(),
            stream: "chat".into(),
            key: Some("room-1".into()),
        };
        let s = serde_json::to_string(&f).unwrap();
        assert!(s.contains("\"type\":\"subscribe\""));
        assert!(s.contains("\"key\":\"room-1\""));
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn subscribe_frame_without_key_omits_field() {
        let f = ClientFrame::Subscribe {
            id: "sub-1".into(),
            stream: "chat".into(),
            key: None,
        };
        let s = serde_json::to_string(&f).unwrap();
        assert!(!s.contains("\"key\""));
    }

    #[test]
    fn publish_frame_roundtrip() {
        let f = ClientFrame::Publish {
            id: "pub-1".into(),
            stream: "chat".into(),
            key: Some("room-1".into()),
            payload: json!({"text": "hi"}),
        };
        let s = serde_json::to_string(&f).unwrap();
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn server_event_frame_parses_with_and_without_key() {
        let with: ServerFrame = serde_json::from_str(
            r#"{"type":"event","id":"sub-1","stream":"chat","key":"room-1","payload":{"x":1}}"#,
        )
        .unwrap();
        let without: ServerFrame = serde_json::from_str(
            r#"{"type":"event","id":"sub-1","stream":"chat","payload":{"x":1}}"#,
        )
        .unwrap();
        match with {
            ServerFrame::Event { key, .. } => assert_eq!(key.as_deref(), Some("room-1")),
            _ => panic!(),
        }
        match without {
            ServerFrame::Event { key, .. } => assert!(key.is_none()),
            _ => panic!(),
        }
    }

    #[test]
    fn error_codes_serialize_snake_case() {
        let cases = [
            (ErrorCode::UnauthorizedPublish, "unauthorized_publish"),
            (
                ErrorCode::PublishPayloadTooLarge,
                "publish_payload_too_large",
            ),
            (ErrorCode::UnknownStream, "unknown_stream"),
            (ErrorCode::Overflow, "overflow"),
        ];
        for (code, want) in cases {
            let s = serde_json::to_string(&code).unwrap();
            assert_eq!(s, format!("\"{}\"", want));
        }
    }
}
