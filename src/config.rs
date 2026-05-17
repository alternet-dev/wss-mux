use std::net::{AddrParseError, SocketAddr};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

use crate::peers::{
    self, PeerUrl, DEFAULT_CLUSTER_DOMAIN, DEFAULT_DNS_REFRESH_SECS, DEFAULT_PEER_SERVICE,
    DEFAULT_RELAY_TIMEOUT_MS,
};

pub const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";
pub const DEFAULT_QUEUE_DEPTH: usize = 1024;
pub const DEFAULT_ENVELOPE_STREAM_PATH: &str = "stream";
pub const DEFAULT_ENVELOPE_KEY_PATH: &str = "key";
pub const DEFAULT_ENVELOPE_PAYLOAD_PATH: &str = "payload";
pub const DEFAULT_INBOUND_RATE: u32 = 50;
pub const DEFAULT_INBOUND_BURST: u32 = 100;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub push_auth_token: String,
    pub handshake_signing_key: String,
    pub manifest_path: PathBuf,
    pub queue_depth: usize,
    /// Dotted path into the push body where the stream name lives.
    /// Default `stream`. Object traversal only (no array indexing); an
    /// empty value means "the whole body".
    pub envelope_stream_path: String,
    /// Dotted path to the optional key. Default `key`.
    pub envelope_key_path: String,
    /// Dotted path to the payload. Default `payload`.
    pub envelope_payload_path: String,
    /// Inbound client-frame rate limit (frames/sec/connection). `0`
    /// disables rate limiting entirely.
    pub inbound_rate_per_sec: u32,
    /// Token-bucket capacity — the largest instantaneous burst allowed
    /// before the steady rate applies.
    pub inbound_burst: u32,
    /// Cross-instance peer-relay discovery + transport settings.
    pub peers: PeerConfig,
}

/// Peer-relay discovery and transport configuration. Every field has a
/// safe default; with no `WSS_MUX_PEER_*` env set the conventional
/// zero-config k8s headless-Service FQDN is composed and resolved, and
/// resolving nothing keeps behaviour byte-identical to a single
/// instance.
#[derive(Debug, Clone)]
pub struct PeerConfig {
    /// `false` only when `WSS_MUX_PEER_RELAY=off` — a hard kill-switch
    /// that also silences the periodic DNS query.
    pub relay_enabled: bool,
    /// Headless Service name for the composed FQDN.
    pub service: String,
    /// Explicit namespace override; `None` ⇒ auto-read from the
    /// ServiceAccount file at discovery time.
    pub namespace_override: Option<String>,
    /// Cluster DNS domain for the composed FQDN.
    pub cluster_domain: String,
    /// Full FQDN override; skips the composed name when set.
    pub peer_dns: Option<String>,
    /// Fixed peer fleet (non-k8s); when non-empty, DNS is not polled.
    pub static_peers: Vec<PeerUrl>,
    /// Per-relay request timeout.
    pub relay_timeout: Duration,
    /// DNS re-resolution interval.
    pub dns_refresh: Duration,
    /// Private CA to trust for `https://` peers.
    pub ca_file: Option<PathBuf>,
    /// Client certificate for mutual TLS to peers.
    pub client_cert: Option<PathBuf>,
    /// Client key for mutual TLS to peers.
    pub client_key: Option<PathBuf>,
}

impl Default for PeerConfig {
    fn default() -> Self {
        PeerConfig {
            relay_enabled: true,
            service: DEFAULT_PEER_SERVICE.to_string(),
            namespace_override: None,
            cluster_domain: DEFAULT_CLUSTER_DOMAIN.to_string(),
            peer_dns: None,
            static_peers: Vec::new(),
            relay_timeout: Duration::from_millis(DEFAULT_RELAY_TIMEOUT_MS),
            dns_refresh: Duration::from_secs(DEFAULT_DNS_REFRESH_SECS),
            ca_file: None,
            client_cert: None,
            client_key: None,
        }
    }
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
    #[error("invalid {var} `{value}`: {source}")]
    InvalidUnsigned {
        var: &'static str,
        value: String,
        #[source]
        source: ParseIntError,
    },
    #[error("invalid WSS_MUX_PEERS: {0}")]
    InvalidPeers(#[source] peers::PeerUrlError),
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

        let envelope_stream_path = get("WSS_MUX_ENVELOPE_STREAM_PATH")
            .unwrap_or_else(|| DEFAULT_ENVELOPE_STREAM_PATH.to_string());
        let envelope_key_path = get("WSS_MUX_ENVELOPE_KEY_PATH")
            .unwrap_or_else(|| DEFAULT_ENVELOPE_KEY_PATH.to_string());
        let envelope_payload_path = get("WSS_MUX_ENVELOPE_PAYLOAD_PATH")
            .unwrap_or_else(|| DEFAULT_ENVELOPE_PAYLOAD_PATH.to_string());

        let parse_u32 = |var: &'static str, default: u32| -> Result<u32, ConfigError> {
            match get(var) {
                Some(v) => v.parse().map_err(|source| ConfigError::InvalidUnsigned {
                    var,
                    value: v.clone(),
                    source,
                }),
                None => Ok(default),
            }
        };
        let inbound_rate_per_sec = parse_u32("WSS_MUX_INBOUND_RATE", DEFAULT_INBOUND_RATE)?;
        let inbound_burst = parse_u32("WSS_MUX_INBOUND_BURST", DEFAULT_INBOUND_BURST)?;

        let parse_u64 = |var: &'static str, default: u64| -> Result<u64, ConfigError> {
            match get(var) {
                Some(v) => v.parse().map_err(|source| ConfigError::InvalidUnsigned {
                    var,
                    value: v.clone(),
                    source,
                }),
                None => Ok(default),
            }
        };
        let non_blank = |v: Option<String>| -> Option<String> {
            v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
        };

        let peers = PeerConfig {
            relay_enabled: match get("WSS_MUX_PEER_RELAY") {
                Some(v) => !v.trim().eq_ignore_ascii_case("off"),
                None => true,
            },
            service: non_blank(get("WSS_MUX_PEER_SERVICE"))
                .unwrap_or_else(|| DEFAULT_PEER_SERVICE.to_string()),
            namespace_override: non_blank(get("WSS_MUX_PEER_NAMESPACE")),
            cluster_domain: non_blank(get("WSS_MUX_CLUSTER_DOMAIN"))
                .unwrap_or_else(|| DEFAULT_CLUSTER_DOMAIN.to_string()),
            peer_dns: non_blank(get("WSS_MUX_PEER_DNS")),
            static_peers: match non_blank(get("WSS_MUX_PEERS")) {
                Some(csv) => peers::parse_static_peers(&csv).map_err(ConfigError::InvalidPeers)?,
                None => Vec::new(),
            },
            relay_timeout: Duration::from_millis(parse_u64(
                "WSS_MUX_PEER_RELAY_TIMEOUT_MS",
                DEFAULT_RELAY_TIMEOUT_MS,
            )?),
            dns_refresh: Duration::from_secs(parse_u64(
                "WSS_MUX_PEER_DNS_REFRESH_SECS",
                DEFAULT_DNS_REFRESH_SECS,
            )?),
            ca_file: non_blank(get("WSS_MUX_PEER_CA_FILE")).map(PathBuf::from),
            client_cert: non_blank(get("WSS_MUX_PEER_CLIENT_CERT")).map(PathBuf::from),
            client_key: non_blank(get("WSS_MUX_PEER_CLIENT_KEY")).map(PathBuf::from),
        };

        Ok(Config {
            listen_addr,
            push_auth_token,
            handshake_signing_key,
            manifest_path,
            queue_depth,
            envelope_stream_path,
            envelope_key_path,
            envelope_payload_path,
            inbound_rate_per_sec,
            inbound_burst,
            peers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peers::Scheme;

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
        assert_eq!(cfg.envelope_stream_path, "stream");
        assert_eq!(cfg.envelope_key_path, "key");
        assert_eq!(cfg.envelope_payload_path, "payload");
    }

    #[test]
    fn envelope_paths_override_from_env() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_ENVELOPE_STREAM_PATH", "meta.topic"));
        pairs.push(("WSS_MUX_ENVELOPE_KEY_PATH", "meta.room"));
        pairs.push(("WSS_MUX_ENVELOPE_PAYLOAD_PATH", "data"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.envelope_stream_path, "meta.topic");
        assert_eq!(cfg.envelope_key_path, "meta.room");
        assert_eq!(cfg.envelope_payload_path, "data");
    }

    #[test]
    fn inbound_rate_defaults_and_overrides() {
        let cfg = Config::from_getter(env(&minimal())).expect("config");
        assert_eq!(cfg.inbound_rate_per_sec, DEFAULT_INBOUND_RATE);
        assert_eq!(cfg.inbound_burst, DEFAULT_INBOUND_BURST);

        let mut pairs = minimal();
        pairs.push(("WSS_MUX_INBOUND_RATE", "0"));
        pairs.push(("WSS_MUX_INBOUND_BURST", "5"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.inbound_rate_per_sec, 0);
        assert_eq!(cfg.inbound_burst, 5);
    }

    #[test]
    fn invalid_inbound_rate_is_error() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_INBOUND_RATE", "fast"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(
            err,
            ConfigError::InvalidUnsigned {
                var: "WSS_MUX_INBOUND_RATE",
                ..
            }
        ));
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

    #[test]
    fn peer_config_defaults_when_unset() {
        let cfg = Config::from_getter(env(&minimal())).expect("config");
        let p = &cfg.peers;
        assert!(p.relay_enabled);
        assert_eq!(p.service, "wss-mux-headless");
        assert_eq!(p.namespace_override, None);
        assert_eq!(p.cluster_domain, "cluster.local");
        assert_eq!(p.peer_dns, None);
        assert!(p.static_peers.is_empty());
        assert_eq!(p.relay_timeout, Duration::from_millis(500));
        assert_eq!(p.dns_refresh, Duration::from_secs(3));
        assert_eq!(p.ca_file, None);
        assert_eq!(p.client_cert, None);
        assert_eq!(p.client_key, None);
    }

    #[test]
    fn peer_config_overrides_from_env() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_PEER_SERVICE", "mux-hl"));
        pairs.push(("WSS_MUX_PEER_NAMESPACE", "team-a"));
        pairs.push(("WSS_MUX_CLUSTER_DOMAIN", "k8s.internal"));
        pairs.push(("WSS_MUX_PEER_DNS", "peers.example.svc"));
        pairs.push(("WSS_MUX_PEERS", "http://a:8080, https://b:9090"));
        pairs.push(("WSS_MUX_PEER_RELAY_TIMEOUT_MS", "750"));
        pairs.push(("WSS_MUX_PEER_DNS_REFRESH_SECS", "10"));
        pairs.push(("WSS_MUX_PEER_CA_FILE", "/etc/ca.pem"));
        pairs.push(("WSS_MUX_PEER_CLIENT_CERT", "/etc/cert.pem"));
        pairs.push(("WSS_MUX_PEER_CLIENT_KEY", "/etc/key.pem"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        let p = &cfg.peers;
        assert_eq!(p.service, "mux-hl");
        assert_eq!(p.namespace_override.as_deref(), Some("team-a"));
        assert_eq!(p.cluster_domain, "k8s.internal");
        assert_eq!(p.peer_dns.as_deref(), Some("peers.example.svc"));
        assert_eq!(
            p.static_peers,
            vec![
                PeerUrl {
                    scheme: Scheme::Http,
                    host: "a".into(),
                    port: 8080
                },
                PeerUrl {
                    scheme: Scheme::Https,
                    host: "b".into(),
                    port: 9090
                },
            ]
        );
        assert_eq!(p.relay_timeout, Duration::from_millis(750));
        assert_eq!(p.dns_refresh, Duration::from_secs(10));
        assert_eq!(
            p.ca_file.as_deref(),
            Some(std::path::Path::new("/etc/ca.pem"))
        );
        assert_eq!(
            p.client_cert.as_deref(),
            Some(std::path::Path::new("/etc/cert.pem"))
        );
        assert_eq!(
            p.client_key.as_deref(),
            Some(std::path::Path::new("/etc/key.pem"))
        );
    }

    #[test]
    fn peer_relay_off_kill_switch() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_PEER_RELAY", "off"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert!(!cfg.peers.relay_enabled);
    }

    #[test]
    fn peer_relay_non_off_value_stays_enabled() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_PEER_RELAY", "on"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert!(cfg.peers.relay_enabled);
    }

    #[test]
    fn invalid_peers_csv_is_error() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_PEERS", "a:8080,bad:port"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::InvalidPeers(_)));
    }

    #[test]
    fn invalid_peer_timeout_is_error() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_PEER_RELAY_TIMEOUT_MS", "soon"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(
            err,
            ConfigError::InvalidUnsigned {
                var: "WSS_MUX_PEER_RELAY_TIMEOUT_MS",
                ..
            }
        ));
    }

    #[test]
    fn blank_peer_overrides_fall_back_to_defaults() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_PEER_SERVICE", ""));
        pairs.push(("WSS_MUX_PEER_NAMESPACE", "  "));
        pairs.push(("WSS_MUX_PEERS", "   "));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.peers.service, "wss-mux-headless");
        assert_eq!(cfg.peers.namespace_override, None);
        assert!(cfg.peers.static_peers.is_empty());
    }
}
