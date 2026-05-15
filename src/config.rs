use std::net::{AddrParseError, SocketAddr};
use std::num::ParseIntError;
use std::path::PathBuf;

use thiserror::Error;

pub const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";
pub const DEFAULT_QUEUE_DEPTH: usize = 1024;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub push_auth_token: String,
    pub handshake_signing_key: String,
    pub manifest_path: PathBuf,
    pub queue_depth: usize,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    Missing(&'static str),
    #[error("invalid WSS_MUX_LISTEN_ADDR `{value}`: {source}")]
    InvalidListenAddr {
        value: String,
        #[source]
        source: AddrParseError,
    },
    #[error("invalid WSS_MUX_QUEUE_DEPTH `{value}`: {source}")]
    InvalidQueueDepth {
        value: String,
        #[source]
        source: ParseIntError,
    },
    #[error("WSS_MUX_QUEUE_DEPTH must be greater than zero")]
    ZeroQueueDepth,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_getter(|k| std::env::var(k).ok())
    }

    fn from_getter<F>(get: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let listen_raw =
            get("WSS_MUX_LISTEN_ADDR").unwrap_or_else(|| DEFAULT_LISTEN_ADDR.to_string());
        let listen_addr = listen_raw
            .parse()
            .map_err(|source| ConfigError::InvalidListenAddr {
                value: listen_raw.clone(),
                source,
            })?;

        let push_auth_token = get("WSS_MUX_PUSH_AUTH_TOKEN")
            .ok_or(ConfigError::Missing("WSS_MUX_PUSH_AUTH_TOKEN"))?;

        let handshake_signing_key = get("WSS_MUX_HANDSHAKE_SIGNING_KEY")
            .ok_or(ConfigError::Missing("WSS_MUX_HANDSHAKE_SIGNING_KEY"))?;

        let manifest_path: PathBuf = get("WSS_MUX_STREAMS_MANIFEST_PATH")
            .ok_or(ConfigError::Missing("WSS_MUX_STREAMS_MANIFEST_PATH"))?
            .into();

        let queue_depth = match get("WSS_MUX_QUEUE_DEPTH") {
            Some(v) => {
                let parsed: usize = v.parse().map_err(|source| ConfigError::InvalidQueueDepth {
                    value: v.clone(),
                    source,
                })?;
                if parsed == 0 {
                    return Err(ConfigError::ZeroQueueDepth);
                }
                parsed
            }
            None => DEFAULT_QUEUE_DEPTH,
        };

        Ok(Config {
            listen_addr,
            push_auth_token,
            handshake_signing_key,
            manifest_path,
            queue_depth,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find_map(|(kk, v)| (*kk == k).then(|| (*v).to_string()))
        }
    }

    fn minimal() -> Vec<(&'static str, &'static str)> {
        vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "push-secret"),
            ("WSS_MUX_HANDSHAKE_SIGNING_KEY", "handshake-secret"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "./streams.yaml"),
        ]
    }

    #[test]
    fn loads_with_minimal_required_env() {
        let cfg = Config::from_getter(env(&minimal())).expect("config");
        assert_eq!(cfg.listen_addr.to_string(), "0.0.0.0:8080");
        assert_eq!(cfg.queue_depth, DEFAULT_QUEUE_DEPTH);
        assert_eq!(cfg.push_auth_token, "push-secret");
        assert_eq!(cfg.handshake_signing_key, "handshake-secret");
        assert_eq!(cfg.manifest_path.to_str(), Some("./streams.yaml"));
    }

    #[test]
    fn missing_push_token_is_error() {
        let pairs = vec![
            ("WSS_MUX_HANDSHAKE_SIGNING_KEY", "h"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
        ];
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(
            err,
            ConfigError::Missing("WSS_MUX_PUSH_AUTH_TOKEN")
        ));
    }

    #[test]
    fn missing_handshake_key_is_error() {
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
        ];
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(
            err,
            ConfigError::Missing("WSS_MUX_HANDSHAKE_SIGNING_KEY")
        ));
    }

    #[test]
    fn missing_manifest_path_is_error() {
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_HANDSHAKE_SIGNING_KEY", "h"),
        ];
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(
            err,
            ConfigError::Missing("WSS_MUX_STREAMS_MANIFEST_PATH")
        ));
    }

    #[test]
    fn invalid_listen_addr_is_error() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_LISTEN_ADDR", "not-an-addr"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::InvalidListenAddr { .. }));
    }

    #[test]
    fn custom_listen_addr_parses() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_LISTEN_ADDR", "127.0.0.1:9090"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.listen_addr.to_string(), "127.0.0.1:9090");
    }

    #[test]
    fn invalid_queue_depth_is_error() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_QUEUE_DEPTH", "not-a-number"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::InvalidQueueDepth { .. }));
    }

    #[test]
    fn zero_queue_depth_is_error() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_QUEUE_DEPTH", "0"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::ZeroQueueDepth));
    }

    #[test]
    fn custom_queue_depth_parses() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_QUEUE_DEPTH", "4096"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.queue_depth, 4096);
    }
}
