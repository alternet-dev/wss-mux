use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
    /// WS-side publish: a connected client emits an event without
    /// dropping back to HTTP `POST /events`. The publish path shares
    /// the same downstream fanout as HTTP push (registry match, peer
    /// relay); the only added surface is the WS frame parse plus the
    /// `producers`-audience check on the connection's principals.
    ///
    /// `id` is a client-chosen, opaque correlation identifier echoed on
    /// any error frame so the client can match failures back to its
    /// originating call. There is no success ack — same ack-by-absence
    /// pattern as subscribe.
    Publish {
        id: String,
        stream: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        payload: Value,
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

/// Where to find each field inside a producer's push body. Dotted paths
/// are object traversal only (no array indexing). An empty path resolves
/// to the whole body.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopePaths<'a> {
    pub stream: &'a str,
    pub key: &'a str,
    pub payload: &'a str,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EnvelopeError {
    #[error("missing or non-string stream at path `{0}`")]
    Stream(String),
    #[error("key at path `{0}` is present but not a string")]
    Key(String),
    #[error("missing payload at path `{0}`")]
    Payload(String),
}

/// Resolve a dotted `path` against `value`, traversing object fields. An
/// empty path returns `value` itself.
pub fn pluck<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }
    let mut cur = value;
    for segment in path.split('.') {
        cur = cur.get(segment)?;
    }
    Some(cur)
}

impl EventEnvelope {
    /// Build an envelope from an arbitrary push body using the configured
    /// paths. With the default paths (`stream`/`key`/`payload`) this is
    /// equivalent to deserializing the body directly into `EventEnvelope`:
    /// stream is a required non-empty string, key is an optional string
    /// (absent or `null` → `None`), payload must be present (explicit
    /// `null` is allowed).
    pub fn from_value(body: &Value, paths: EnvelopePaths<'_>) -> Result<Self, EnvelopeError> {
        let stream = pluck(body, paths.stream)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| EnvelopeError::Stream(paths.stream.to_string()))?
            .to_string();

        let key = match pluck(body, paths.key) {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) => Some(s.clone()),
            Some(_) => return Err(EnvelopeError::Key(paths.key.to_string())),
        };

        let payload = pluck(body, paths.payload)
            .cloned()
            .ok_or_else(|| EnvelopeError::Payload(paths.payload.to_string()))?;

        Ok(EventEnvelope {
            stream,
            key,
            payload,
        })
    }
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
    fn publish_with_key_roundtrips() {
        let frame = ClientFrame::Publish {
            id: "p1".into(),
            stream: "chat_messages".into(),
            key: Some("room-42".into()),
            payload: json!({"from": "alice", "text": "hi"}),
        };
        let s = serde_json::to_string(&frame).unwrap();
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn publish_without_key_omits_field() {
        let frame = ClientFrame::Publish {
            id: "p1".into(),
            stream: "presence".into(),
            key: None,
            payload: json!({"online": true}),
        };
        let s = serde_json::to_string(&frame).unwrap();
        assert!(!s.contains("\"key\""));
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn publish_missing_key_field_parses_as_none() {
        let raw = r#"{"type":"publish","id":"p1","stream":"presence","payload":{"online":true}}"#;
        let frame: ClientFrame = serde_json::from_str(raw).unwrap();
        assert_eq!(
            frame,
            ClientFrame::Publish {
                id: "p1".into(),
                stream: "presence".into(),
                key: None,
                payload: json!({"online": true}),
            }
        );
    }

    #[test]
    fn publish_null_payload_is_allowed() {
        // Mirrors EventEnvelope's "explicit null payload is allowed"
        // semantics — a publish carrying `payload: null` round-trips.
        let frame = ClientFrame::Publish {
            id: "p1".into(),
            stream: "presence".into(),
            key: None,
            payload: Value::Null,
        };
        let s = serde_json::to_string(&frame).unwrap();
        let back: ClientFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn publish_missing_payload_fails_to_parse() {
        let raw = r#"{"type":"publish","id":"p1","stream":"presence"}"#;
        let res: Result<ClientFrame, _> = serde_json::from_str(raw);
        assert!(res.is_err(), "payload is required on publish");
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

    const DEFAULTS: EnvelopePaths<'static> = EnvelopePaths {
        stream: "stream",
        key: "key",
        payload: "payload",
    };

    #[test]
    fn pluck_traverses_nested_objects() {
        let v = json!({"meta": {"topic": "chat", "n": 1}});
        assert_eq!(pluck(&v, "meta.topic"), Some(&json!("chat")));
        assert_eq!(pluck(&v, "meta.n"), Some(&json!(1)));
    }

    #[test]
    fn pluck_misses_return_none() {
        let v = json!({"meta": {"topic": "chat"}});
        assert_eq!(pluck(&v, "meta.missing"), None);
        assert_eq!(pluck(&v, "nope.topic"), None);
        // Mid-path is not an object.
        assert_eq!(pluck(&v, "meta.topic.deeper"), None);
    }

    #[test]
    fn pluck_empty_path_is_identity() {
        let v = json!({"a": 1});
        assert_eq!(pluck(&v, ""), Some(&v));
    }

    #[test]
    fn from_value_defaults_match_direct_deserialization() {
        let body = json!({"stream": "chat", "key": "room-1", "payload": {"x": 1}});
        let env = EventEnvelope::from_value(&body, DEFAULTS).expect("envelope");
        let direct: EventEnvelope = serde_json::from_value(body).unwrap();
        assert_eq!(env, direct);
    }

    #[test]
    fn from_value_absent_and_null_key_are_none() {
        let absent = json!({"stream": "chat", "payload": {}});
        assert_eq!(
            EventEnvelope::from_value(&absent, DEFAULTS).unwrap().key,
            None
        );
        let null = json!({"stream": "chat", "key": null, "payload": {}});
        assert_eq!(
            EventEnvelope::from_value(&null, DEFAULTS).unwrap().key,
            None
        );
    }

    #[test]
    fn from_value_custom_paths() {
        let paths = EnvelopePaths {
            stream: "meta.topic",
            key: "meta.room",
            payload: "data",
        };
        let body = json!({
            "meta": {"topic": "chat_messages", "room": "42"},
            "data": {"text": "hi"}
        });
        let env = EventEnvelope::from_value(&body, paths).expect("envelope");
        assert_eq!(env.stream, "chat_messages");
        assert_eq!(env.key.as_deref(), Some("42"));
        assert_eq!(env.payload, json!({"text": "hi"}));
    }

    #[test]
    fn from_value_missing_stream_errors() {
        let body = json!({"payload": {}});
        assert_eq!(
            EventEnvelope::from_value(&body, DEFAULTS),
            Err(EnvelopeError::Stream("stream".into()))
        );
    }

    #[test]
    fn from_value_non_string_stream_errors() {
        let body = json!({"stream": 7, "payload": {}});
        assert!(matches!(
            EventEnvelope::from_value(&body, DEFAULTS),
            Err(EnvelopeError::Stream(_))
        ));
    }

    #[test]
    fn from_value_non_string_key_errors() {
        let body = json!({"stream": "chat", "key": 7, "payload": {}});
        assert!(matches!(
            EventEnvelope::from_value(&body, DEFAULTS),
            Err(EnvelopeError::Key(_))
        ));
    }

    #[test]
    fn from_value_missing_payload_errors() {
        let body = json!({"stream": "chat"});
        assert!(matches!(
            EventEnvelope::from_value(&body, DEFAULTS),
            Err(EnvelopeError::Payload(_))
        ));
        // Explicit null payload is allowed.
        let with_null = json!({"stream": "chat", "payload": null});
        assert_eq!(
            EventEnvelope::from_value(&with_null, DEFAULTS)
                .unwrap()
                .payload,
            Value::Null
        );
    }
}
