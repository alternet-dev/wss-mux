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
