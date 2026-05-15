use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientFrame {
    Auth {
        token: String,
    },
    Subscribe {
        id: String,
        stream: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
    },
    Unsubscribe {
        id: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerFrame {
    Event {
        id: String,
        stream: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        payload: Value,
    },
    Error {
        code: String,
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EventEnvelope {
    pub stream: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub payload: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn auth_roundtrips() {
        let frame = ClientFrame::Auth {
            token: "abc".into(),
        };
        let s = serde_json::to_string(&frame).unwrap();
        assert_eq!(s, r#"{"type":"auth","token":"abc"}"#);
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn subscribe_with_key_roundtrips() {
        let frame = ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
        };
        let s = serde_json::to_string(&frame).unwrap();
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn subscribe_without_key_omits_field() {
        let frame = ClientFrame::Subscribe {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: None,
        };
        let s = serde_json::to_string(&frame).unwrap();
        assert!(!s.contains("key"));
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn subscribe_missing_key_field_parses_as_none() {
        let raw = r#"{"type":"subscribe","id":"s1","stream":"chat_messages"}"#;
        let frame: ClientFrame = serde_json::from_str(raw).unwrap();
        assert_eq!(
            frame,
            ClientFrame::Subscribe {
                id: "s1".into(),
                stream: "chat_messages".into(),
                key: None
            }
        );
    }

    #[test]
    fn unsubscribe_roundtrips() {
        let frame = ClientFrame::Unsubscribe { id: "s1".into() };
        let s = serde_json::to_string(&frame).unwrap();
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn unknown_type_fails_to_parse() {
        let raw = r#"{"type":"nope"}"#;
        let res: Result<ClientFrame, _> = serde_json::from_str(raw);
        assert!(res.is_err());
    }

    #[test]
    fn event_frame_roundtrips() {
        let frame = ServerFrame::Event {
            id: "s1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
            payload: json!({"from": "alice", "text": "hi"}),
        };
        let s = serde_json::to_string(&frame).unwrap();
        let back: ServerFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn error_frame_with_id_roundtrips() {
        let frame = ServerFrame::Error {
            code: "unknown_stream".into(),
            message: "no such stream".into(),
            id: Some("s1".into()),
        };
        let s = serde_json::to_string(&frame).unwrap();
        assert!(s.contains("\"id\":\"s1\""));
        let back: ServerFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn error_frame_without_id_omits_field() {
        let frame = ServerFrame::Error {
            code: "bad_frame".into(),
            message: "parse failed".into(),
            id: None,
        };
        let s = serde_json::to_string(&frame).unwrap();
        assert!(!s.contains("\"id\""));
    }

    #[test]
    fn event_envelope_roundtrips() {
        let envelope = EventEnvelope {
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
            payload: json!({"text": "hi"}),
        };
        let s = serde_json::to_string(&envelope).unwrap();
        let back: EventEnvelope = serde_json::from_str(&s).unwrap();
        assert_eq!(back, envelope);
    }

    #[test]
    fn event_envelope_without_key_roundtrips() {
        let envelope = EventEnvelope {
            stream: "presence".into(),
            key: None,
            payload: json!({"who": "alice"}),
        };
        let s = serde_json::to_string(&envelope).unwrap();
        assert!(!s.contains("\"key\""));
        let back: EventEnvelope = serde_json::from_str(&s).unwrap();
        assert_eq!(back, envelope);
    }
}
