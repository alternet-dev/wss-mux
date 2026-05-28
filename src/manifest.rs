use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub version: u32,
    pub streams: Vec<Stream>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Stream {
    #[serde(rename = "stream")]
    pub name: String,
    /// Principals admitted to the `subscribe` action on this stream.
    /// Matching rules: exact match, trailing-`*` prefix, or the bare
    /// `*` (which admits any authenticated connection).
    pub subscribe: Vec<String>,
    /// Principals admitted to the WS `publish` action on this stream.
    /// Default empty ⇒ no one may WS-publish here (explicit opt-in
    /// per stream). HTTP `POST /events` is unaffected — it uses the
    /// shared `WSS_MUX_PUSH_AUTH_TOKEN`, not principal audience.
    /// Same matching rules as `subscribe`.
    #[serde(default)]
    pub publish: Vec<String>,
    /// Optional per-stream payload cap, measured as the length of the
    /// JSON serialization of the event payload (stable regardless of
    /// the producer/relay wire codec). Absent ⇒ no cap. A push whose
    /// payload exceeds it is rejected; in a batch, one oversized event
    /// rejects the whole batch. The same cap applies to WS publish.
    #[serde(default)]
    pub max_payload_bytes: Option<u64>,
    /// Optional per-stream send-queue depth override. Absent ⇒ the
    /// global `WSS_MUX_QUEUE_DEPTH`. This is the max in-flight frames
    /// for a single subscription on this stream before that
    /// subscription overflows (and is dropped on its own).
    #[serde(default)]
    pub queue_depth: Option<usize>,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read manifest at {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse manifest at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("unsupported manifest version {0} (only version 1 is supported)")]
    UnsupportedVersion(u32),
    #[error("stream `{0}` has an empty `subscribe` audience")]
    EmptySubscribe(String),
    #[error("duplicate stream name `{0}`")]
    DuplicateStream(String),
    #[error(
        "stream `{stream}` audience entry `{entry}` may only use `*` as a single trailing wildcard"
    )]
    InvalidWildcard { stream: String, entry: String },
    // (The error term "audience entry" here refers generically to a
    // principal pattern used in either `subscribe` or `publish`.)
    #[error("stream `{0}` has max_payload_bytes: 0, which rejects every event")]
    ZeroMaxPayload(String),
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path).map_err(|source| ManifestError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_str(&content, path)
    }

    pub fn from_str(content: &str, path: &Path) -> Result<Self, ManifestError> {
        let manifest: Manifest =
            serde_yaml::from_str(content).map_err(|source| ManifestError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.version != 1 {
            return Err(ManifestError::UnsupportedVersion(self.version));
        }
        let mut seen = HashSet::new();
        for stream in &self.streams {
            if stream.subscribe.is_empty() {
                return Err(ManifestError::EmptySubscribe(stream.name.clone()));
            }
            // Same wildcard rules apply to `subscribe` and `publish`:
            // a single trailing `*` is the only star allowed.
            for entry in stream.subscribe.iter().chain(stream.publish.iter()) {
                let stars = entry.matches('*').count();
                if stars > 1 || (stars == 1 && !entry.ends_with('*')) {
                    return Err(ManifestError::InvalidWildcard {
                        stream: stream.name.clone(),
                        entry: entry.clone(),
                    });
                }
            }
            if stream.max_payload_bytes == Some(0) {
                // A 0 cap rejects every conceivable payload — almost
                // certainly a misconfiguration. Fail fast rather than
                // silently black-hole the stream.
                return Err(ManifestError::ZeroMaxPayload(stream.name.clone()));
            }
            if !seen.insert(stream.name.as_str()) {
                return Err(ManifestError::DuplicateStream(stream.name.clone()));
            }
        }
        Ok(())
    }

    pub fn stream(&self, name: &str) -> Option<&Stream> {
        self.streams.iter().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
  - stream: presence
    subscribe: [role:member, role:operator]
"#;

    fn p() -> &'static Path {
        Path::new("manifest.yaml")
    }

    #[test]
    fn parses_valid_manifest() {
        let m = Manifest::from_str(VALID, p()).expect("manifest");
        assert_eq!(m.version, 1);
        assert_eq!(m.streams.len(), 2);
        assert_eq!(m.streams[0].name, "chat_messages");
        assert_eq!(m.streams[0].subscribe, vec!["role:member".to_string()]);
        assert_eq!(m.streams[1].name, "presence");
        assert_eq!(m.streams[1].subscribe.len(), 2);
    }

    #[test]
    fn rejects_wrong_version() {
        let s = "version: 2\nstreams: []\n";
        let err = Manifest::from_str(s, p()).expect_err("error");
        assert!(matches!(err, ManifestError::UnsupportedVersion(2)));
    }

    #[test]
    fn rejects_empty_subscribe() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: []
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        match err {
            ManifestError::EmptySubscribe(name) => assert_eq!(name, "chat_messages"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rejects_duplicate_stream_name() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
  - stream: chat_messages
    subscribe: [role:operator]
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        match err {
            ManifestError::DuplicateStream(name) => assert_eq!(name, "chat_messages"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn accepts_trailing_and_bare_wildcards() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:*]
  - stream: public_feed
    subscribe: ["*"]
  - stream: tenant_events
    subscribe: [tenant:*, role:operator]
"#;
        let m = Manifest::from_str(s, p()).expect("manifest");
        assert_eq!(m.streams.len(), 3);
        assert_eq!(m.streams[0].subscribe, vec!["role:*".to_string()]);
        assert_eq!(m.streams[1].subscribe, vec!["*".to_string()]);
    }

    #[test]
    fn rejects_wildcard_not_in_final_position() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [ro*le]
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        match err {
            ManifestError::InvalidWildcard { stream, entry } => {
                assert_eq!(stream, "chat_messages");
                assert_eq!(entry, "ro*le");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rejects_leading_wildcard() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: ["*member"]
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        assert!(matches!(err, ManifestError::InvalidWildcard { .. }));
    }

    #[test]
    fn rejects_multiple_wildcards() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: ["a*b*"]
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        assert!(matches!(err, ManifestError::InvalidWildcard { .. }));
    }

    #[test]
    fn parses_optional_max_payload_bytes() {
        let s = r#"
version: 1
streams:
  - stream: capped
    subscribe: [role:member]
    max_payload_bytes: 4096
  - stream: uncapped
    subscribe: [role:member]
"#;
        let m = Manifest::from_str(s, p()).expect("manifest");
        assert_eq!(m.stream("capped").unwrap().max_payload_bytes, Some(4096));
        assert_eq!(m.stream("uncapped").unwrap().max_payload_bytes, None);
    }

    #[test]
    fn rejects_zero_max_payload() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    max_payload_bytes: 0
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        match err {
            ManifestError::ZeroMaxPayload(name) => assert_eq!(name, "chat_messages"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn publish_defaults_to_empty_when_absent() {
        // Absent `publish` ⇒ no one may WS-publish to the stream
        // (default-deny). Manifest still parses; HTTP push is unaffected.
        let m = Manifest::from_str(VALID, p()).expect("manifest");
        assert!(m.streams[0].publish.is_empty());
        assert!(m.streams[1].publish.is_empty());
    }

    #[test]
    fn parses_explicit_publish_audience() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    publish: [role:member]
  - stream: presence
    subscribe: [role:member, role:operator]
    publish: [role:member]
"#;
        let m = Manifest::from_str(s, p()).expect("manifest");
        assert_eq!(
            m.stream("chat_messages").unwrap().publish,
            vec!["role:member".to_string()]
        );
        assert_eq!(
            m.stream("presence").unwrap().publish,
            vec!["role:member".to_string()]
        );
    }

    #[test]
    fn publish_wildcard_rules_match_subscribe() {
        // Trailing-star and bare-star are accepted just like audience.
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    publish: [role:*]
  - stream: open_feed
    subscribe: ["*"]
    publish: ["*"]
"#;
        let m = Manifest::from_str(s, p()).expect("manifest");
        assert_eq!(
            m.stream("chat_messages").unwrap().publish,
            vec!["role:*".to_string()]
        );
        assert_eq!(
            m.stream("open_feed").unwrap().publish,
            vec!["*".to_string()]
        );
    }

    #[test]
    fn rejects_invalid_wildcard_in_publish() {
        // Same rejection as for audience — a star anywhere but a single
        // trailing position is rejected at load.
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    publish: ["ro*le"]
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        match err {
            ManifestError::InvalidWildcard { stream, entry } => {
                assert_eq!(stream, "chat_messages");
                assert_eq!(entry, "ro*le");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parses_optional_queue_depth() {
        let s = r#"
version: 1
streams:
  - stream: bursty
    subscribe: [role:member]
    queue_depth: 64
  - stream: defaulted
    subscribe: [role:member]
"#;
        let m = Manifest::from_str(s, p()).expect("manifest");
        assert_eq!(m.stream("bursty").unwrap().queue_depth, Some(64));
        assert_eq!(m.stream("defaulted").unwrap().queue_depth, None);
    }

    #[test]
    fn zero_queue_depth_means_unlimited() {
        // `0` is the explicit "unlimited" signal for the stream (the
        // dispatcher skips the per-subscription cap); it parses rather
        // than being rejected at load.
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    queue_depth: 0
"#;
        let m = Manifest::from_str(s, p()).expect("valid manifest");
        assert_eq!(m.stream("chat_messages").unwrap().queue_depth, Some(0));
    }

    // `queue_depth`/`max_payload_bytes` are unsigned (`usize`/`u64`), so
    // a negative value is unrepresentable: serde rejects it at YAML
    // parse and the whole manifest is refused at load/hot-reload. These
    // lock that `< 0` can never silently become `0`, a huge wrapped
    // value, or a usable cap.
    #[test]
    fn rejects_negative_queue_depth_at_parse() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    queue_depth: -1
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        assert!(
            matches!(err, ManifestError::Parse { .. }),
            "negative queue_depth must be rejected at parse, got {err:?}"
        );
    }

    #[test]
    fn rejects_negative_max_payload_bytes_at_parse() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    max_payload_bytes: -1
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        assert!(
            matches!(err, ManifestError::Parse { .. }),
            "negative max_payload_bytes must be rejected at parse, got {err:?}"
        );
    }

    #[test]
    fn rejects_malformed_yaml() {
        let s = "not: : : valid";
        let err = Manifest::from_str(s, p()).expect_err("error");
        assert!(matches!(err, ManifestError::Parse { .. }));
    }

    #[test]
    fn load_returns_read_error_for_missing_file() {
        let bogus = std::env::temp_dir().join("wss-mux-test-this-file-does-not-exist-9f3a.yaml");
        let err = Manifest::load(&bogus).expect_err("error");
        assert!(matches!(err, ManifestError::Read { .. }));
    }

    #[test]
    fn stream_lookup() {
        let m = Manifest::from_str(VALID, p()).expect("manifest");
        assert_eq!(m.stream("chat_messages").unwrap().name, "chat_messages");
        assert!(m.stream("nope").is_none());
    }
}
