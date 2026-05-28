use crate::connection::Outbound;
use crate::envelope::ServerFrame;

/// One variant per row of the error code table in `docs/protocol.md`.
///
/// `Overflow` is keep-open and per-subscription (v0.4): the offending
/// subscription is dropped, the connection and its other subscriptions
/// survive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    UnknownFrameType,
    BadFrame,
    Unauthenticated { id: Option<String> },
    ExpiredToken,
    UnknownStream { id: String },
    UnauthorizedSubscribe { id: String },
    UnauthorizedPublish { id: String },
    DuplicateSubscriptionId { id: String },
    PublishPayloadTooLarge { id: String },
    RateLimited { id: Option<String> },
    Overflow { id: String },
}

impl ProtocolError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownFrameType => "unknown_frame_type",
            Self::BadFrame => "bad_frame",
            Self::Unauthenticated { .. } => "unauthenticated",
            Self::ExpiredToken => "expired_token",
            Self::UnknownStream { .. } => "unknown_stream",
            Self::UnauthorizedSubscribe { .. } => "unauthorized_subscribe",
            Self::UnauthorizedPublish { .. } => "unauthorized_publish",
            Self::DuplicateSubscriptionId { .. } => "duplicate_subscription_id",
            Self::PublishPayloadTooLarge { .. } => "publish_payload_too_large",
            Self::RateLimited { .. } => "rate_limited",
            Self::Overflow { .. } => "overflow",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownFrameType => "unrecognized frame type",
            Self::BadFrame => "frame failed parsing or schema",
            Self::Unauthenticated { .. } => "non-auth frame before auth",
            Self::ExpiredToken => "auth token has expired",
            Self::UnknownStream { .. } => "stream not declared in manifest",
            Self::UnauthorizedSubscribe { .. } => {
                "principals do not intersect stream subscribe audience"
            }
            Self::UnauthorizedPublish { .. } => {
                "principals do not intersect stream publish audience"
            }
            Self::DuplicateSubscriptionId { .. } => "subscription id already in use",
            Self::PublishPayloadTooLarge { .. } => {
                "publish payload exceeds the stream's max_payload_bytes cap"
            }
            Self::RateLimited { .. } => "inbound frame rate limit exceeded",
            Self::Overflow { .. } => "per-subscription send queue overflowed",
        }
    }

    /// Per-frame correlation id when the error pertains to a specific frame
    /// (`docs/protocol.md` says: "`id` is included when the error pertains to
    /// a specific frame, absent otherwise").
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Unauthenticated { id } | Self::RateLimited { id } => id.as_deref(),
            Self::UnknownStream { id }
            | Self::UnauthorizedSubscribe { id }
            | Self::UnauthorizedPublish { id }
            | Self::DuplicateSubscriptionId { id }
            | Self::PublishPayloadTooLarge { id }
            | Self::Overflow { id } => Some(id),
            _ => None,
        }
    }

    /// WebSocket close code when the error closes the connection. None when
    /// the connection stays open after the error frame.
    pub fn close_code(&self) -> Option<u16> {
        match self {
            Self::UnknownFrameType | Self::BadFrame => Some(4400),
            Self::Unauthenticated { .. } | Self::ExpiredToken => Some(4401),
            _ => None,
        }
    }

    pub fn closes_connection(&self) -> bool {
        self.close_code().is_some()
    }

    pub fn to_frame(&self) -> ServerFrame {
        ServerFrame::Error {
            code: self.code().to_string(),
            message: self.message().to_string(),
            id: self.id().map(str::to_string),
        }
    }

    /// Produce the `Outbound` the reader should hand to the writer. Closing
    /// errors carry the error frame as the trailing message before close;
    /// keep-open errors are sent as a normal frame.
    pub fn to_outbound(&self) -> Outbound {
        let frame = self.to_frame();
        match self.close_code() {
            Some(code) => Outbound::Close {
                code,
                reason: self.code().to_string(),
                frame: Some(frame),
            },
            None => Outbound::Frame(frame),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_codes_match_protocol_md() {
        assert_eq!(ProtocolError::UnknownFrameType.close_code(), Some(4400));
        assert_eq!(ProtocolError::BadFrame.close_code(), Some(4400));
        assert_eq!(
            ProtocolError::Unauthenticated { id: None }.close_code(),
            Some(4401)
        );
        assert_eq!(ProtocolError::ExpiredToken.close_code(), Some(4401));
        assert_eq!(
            ProtocolError::UnknownStream { id: "x".into() }.close_code(),
            None
        );
        assert_eq!(
            ProtocolError::UnauthorizedSubscribe { id: "x".into() }.close_code(),
            None
        );
        assert_eq!(
            ProtocolError::DuplicateSubscriptionId { id: "x".into() }.close_code(),
            None
        );
        assert_eq!(ProtocolError::RateLimited { id: None }.close_code(), None);
    }

    #[test]
    fn rate_limited_is_keep_open_and_echoes_id() {
        let err = ProtocolError::RateLimited {
            id: Some("s9".into()),
        };
        assert_eq!(err.code(), "rate_limited");
        assert!(matches!(err.to_outbound(), Outbound::Frame(_)));
        let ServerFrame::Error { id, .. } = err.to_frame() else {
            unreachable!()
        };
        assert_eq!(id.as_deref(), Some("s9"));
    }

    #[test]
    fn overflow_is_keep_open_and_echoes_sub_id() {
        // v0.4: overflow is per-subscription and keep-open — the
        // offending subscription is dropped, the connection survives.
        let err = ProtocolError::Overflow { id: "s1".into() };
        assert_eq!(err.code(), "overflow");
        assert_eq!(
            err.close_code(),
            None,
            "overflow no longer closes the connection"
        );
        assert_eq!(err.id(), Some("s1"));
        assert!(matches!(err.to_outbound(), Outbound::Frame(_)));
        let ServerFrame::Error { id, .. } = err.to_frame() else {
            unreachable!()
        };
        assert_eq!(id.as_deref(), Some("s1"));
    }

    #[test]
    fn keep_open_errors_emit_frame_outbound() {
        let err = ProtocolError::UnknownStream { id: "s1".into() };
        assert!(matches!(err.to_outbound(), Outbound::Frame(_)));
    }

    #[test]
    fn closing_errors_emit_close_outbound_with_trailing_frame() {
        let err = ProtocolError::BadFrame;
        match err.to_outbound() {
            Outbound::Close { code, frame, .. } => {
                assert_eq!(code, 4400);
                assert!(frame.is_some());
            }
            other => panic!("expected Close, got {other:?}"),
        }
    }

    #[test]
    fn frame_carries_id_when_applicable() {
        let err = ProtocolError::UnknownStream { id: "s7".into() };
        let ServerFrame::Error { id, .. } = err.to_frame() else {
            unreachable!()
        };
        assert_eq!(id.as_deref(), Some("s7"));
    }

    #[test]
    fn frame_omits_id_for_connection_level_errors() {
        let err = ProtocolError::ExpiredToken;
        let ServerFrame::Error { id, .. } = err.to_frame() else {
            unreachable!()
        };
        assert_eq!(id, None);
    }

    #[test]
    fn unauthenticated_carries_id_when_set() {
        let err = ProtocolError::Unauthenticated {
            id: Some("s1".into()),
        };
        assert_eq!(err.id(), Some("s1"));
    }
}
