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
    pub audience: Vec<String>,
    /// Optional per-stream payload cap, measured as the length of the
    /// JSON serialization of the event payload (stable regardless of
    /// the producer/relay wire codec). Absent ⇒ no cap. A push whose
    /// payload exceeds it is rejected; in a batch, one oversized event
    /// rejects the whole batch.
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
    #[error("stream `{0}` has an empty audience")]
    EmptyAudience(String),
    #[error("duplicate stream name `{0}`")]
    DuplicateStream(String),
    #[error(
        "stream `{stream}` audience entry `{entry}` may only use `*` as a single trailing wildcard"
    )]
    InvalidWildcard { stream: String, entry: String },
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
            if stream.audience.is_empty() {
                return Err(ManifestError::EmptyAudience(stream.name.clone()));
            }
            for entry in &stream.audience {
                // A `*` is only meaningful as a single trailing
                // wildcard (or the bare `*`). Reject any other use so
                // the prefix-match semantics stay unambiguous.
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
    audience: [role:member]
  - stream: presence
    audience: [role:member, role:operator]
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
        assert_eq!(m.streams[0].audience, vec!["role:member".to_string()]);
        assert_eq!(m.streams[1].name, "presence");
        assert_eq!(m.streams[1].audience.len(), 2);
    }

    #[test]
    fn rejects_wrong_version() {
        let s = "version: 2\nstreams: []\n";
        let err = Manifest::from_str(s, p()).expect_err("error");
        assert!(matches!(err, ManifestError::UnsupportedVersion(2)));
    }

    #[test]
    fn rejects_empty_audience() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    audience: []
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        match err {
            ManifestError::EmptyAudience(name) => assert_eq!(name, "chat_messages"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rejects_duplicate_stream_name() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    audience: [role:member]
  - stream: chat_messages
    audience: [role:operator]
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
    audience: [role:*]
  - stream: public_feed
    audience: ["*"]
  - stream: tenant_events
    audience: [tenant:*, role:operator]
"#;
        let m = Manifest::from_str(s, p()).expect("manifest");
        assert_eq!(m.streams.len(), 3);
        assert_eq!(m.streams[0].audience, vec!["role:*".to_string()]);
        assert_eq!(m.streams[1].audience, vec!["*".to_string()]);
    }

    #[test]
    fn rejects_wildcard_not_in_final_position() {
        let s = r#"
version: 1
streams:
  - stream: chat_messages
    audience: [ro*le]
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
    audience: ["*member"]
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
    audience: ["a*b*"]
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
    audience: [role:member]
    max_payload_bytes: 4096
  - stream: uncapped
    audience: [role:member]
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
    audience: [role:member]
    max_payload_bytes: 0
"#;
        let err = Manifest::from_str(s, p()).expect_err("error");
        match err {
            ManifestError::ZeroMaxPayload(name) => assert_eq!(name, "chat_messages"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parses_optional_queue_depth() {
        let s = r#"
version: 1
streams:
  - stream: bursty
    audience: [role:member]
    queue_depth: 64
  - stream: defaulted
    audience: [role:member]
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
    audience: [role:member]
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
    audience: [role:member]
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
    audience: [role:member]
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
