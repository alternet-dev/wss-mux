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
/// Per-source WS publish rate (frames/sec/source). `0` disables the
/// per-source limit entirely (no `PerSourceRateLimiter` is constructed,
/// no eviction task is spawned, byte-identical to behaviour pre-feature).
pub const DEFAULT_WS_PUBLISH_RATE: u32 = 0;
/// Per-source idle-bucket eviction TTL. The sweep runs at one tenth
/// this cadence; entries whose last touch is older than the TTL are
/// dropped to bound the per-source bucket map.
pub const DEFAULT_WS_PUBLISH_IDLE_TTL_SECS: u64 = 300;
/// When `false`, a token with an empty `sub` falls back to a
/// `conn:<id>` source key (per-connection limit instead of per-user).
/// When `true`, such a token is refused with `unauthorized_publish`.
pub const DEFAULT_WS_PUBLISH_REQUIRE_SUB: bool = false;
pub const DEFAULT_RELAY_COALESCE_MS: u64 = 0;
pub const DEFAULT_RELAY_COALESCE_MAX_EVENTS: usize = 1024;
pub const DEFAULT_RELAY_QUEUE_DEPTH: usize = 1024;
pub const DEFAULT_OIDC_GROUPS_CLAIM: &str = "groups";
pub const DEFAULT_OIDC_JWKS_REFRESH_SECS: u64 = 300;

/// Handshake-token verification keys. At least one of HS256 / Ed25519
/// must be configured; both is allowed (the token's `alg` then selects
/// *which configured key* to verify against — it never widens the set
/// of accepted algorithms).
#[derive(Debug, Clone)]
pub struct HandshakeKeyConfig {
    /// HS256 shared secret (`WSS_MUX_HANDSHAKE_SIGNING_KEY`).
    pub hs256_secret: Option<String>,
    /// Ed25519 public key, PEM. From `WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY`
    /// (inline) or `…_FILE` (path to a PEM file).
    pub ed25519_public_pem: Option<String>,
}

/// Optional OIDC validation. Present only when `WSS_MUX_OIDC_ISSUER`
/// is set, in which case the auth-frame token is validated *solely*
/// against the IdP — mutually exclusive with the handshake key (one
/// trust root, no issuer/alg-confusion surface). Unset ⇒ inert,
/// behaviour byte-identical to handshake-only.
#[derive(Debug, Clone)]
pub struct OidcConfig {
    /// `WSS_MUX_OIDC_ISSUER` — the IdP issuer URL; also the `iss` the
    /// token must carry. Used for strict token-iss validation and, by
    /// default, as the discovery base.
    pub issuer: String,
    /// `WSS_MUX_OIDC_AUDIENCE` — required: the `aud` the token must
    /// carry (pinning an audience is mandatory; accepting any
    /// audience is unsafe).
    pub audience: String,
    /// `WSS_MUX_OIDC_DISCOVERY_URL` — base for the OIDC discovery
    /// document. When present, `<discovery_url>/.well-known/openid-configuration`
    /// is fetched; otherwise the base is `issuer`. Lets the public
    /// `iss` value (what tokens carry) diverge from the in-cluster
    /// endpoint wss-mux actually reaches — the Keycloak
    /// `KC_HOSTNAME_BACKCHANNEL_DYNAMIC` pattern, etc.
    pub discovery_url: Option<String>,
    /// `WSS_MUX_OIDC_JWKS_URL` — explicit JWKS endpoint; absent ⇒
    /// discovered from `<discovery_url|issuer>/.well-known/openid-configuration`.
    pub jwks_url: Option<String>,
    /// `WSS_MUX_OIDC_GROUPS_CLAIM` (default `groups`) — array claim
    /// whose values become principals.
    pub groups_claim: String,
    /// `WSS_MUX_OIDC_PRINCIPAL_PREFIX` (default empty) — prefix applied
    /// to each group value (e.g. `role:`) so IdP groups line up with
    /// manifest audiences.
    pub principal_prefix: String,
    /// `WSS_MUX_OIDC_JWKS_REFRESH` seconds (default 300).
    pub jwks_refresh: Duration,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub push_auth_token: String,
    pub handshake_keys: HandshakeKeyConfig,
    /// OIDC validation, when `WSS_MUX_OIDC_ISSUER` is set (mutually
    /// exclusive with `handshake_keys`).
    pub oidc: Option<OidcConfig>,
    pub manifest_path: PathBuf,
    /// Optional shared bearer that authenticates the SSE read endpoint
    /// (`GET /events/:stream`). When set, a request whose
    /// `Authorization: Bearer <token>` matches is admitted without
    /// any principal/audience check — full read access across streams,
    /// matching the posture of `WSS_MUX_PUSH_AUTH_TOKEN` on the
    /// produce side. When unset, the SSE endpoint accepts only JWTs
    /// (and only when the connection's principals intersect the
    /// stream's `subscribe` audience). Both can coexist; either path
    /// alone admits.
    pub read_auth_token: Option<String>,
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
    /// Per-source WS publish rate limit (frames/sec/source). `0`
    /// disables the per-source publish limit entirely. Source is the
    /// connection's JWT `sub` claim — with a `conn:<id>` fallback when
    /// `sub` is empty, unless `ws_publish_require_sub` rejects that
    /// case outright.
    pub ws_publish_rate_per_sec: u32,
    /// Token-bucket capacity for the per-source WS publish limiter.
    /// `0` ⇒ defaults to `2 * ws_publish_rate_per_sec` (per env-parse).
    pub ws_publish_burst: u32,
    /// When `true`, a connection whose token carries no `sub` (or an
    /// empty `sub`) is refused at publish time with `unauthorized_publish`.
    /// When `false` (default), such a connection falls back to a
    /// per-connection source key (`conn:<id>`).
    pub ws_publish_require_sub: bool,
    /// Idle-bucket eviction TTL for the per-source WS publish limiter.
    /// A bucket whose `last_seen` is older than this is dropped; the
    /// sweep runs at one-tenth this cadence on a tokio interval task.
    pub ws_publish_idle_ttl: Duration,
    /// Relay coalescing flush window (ms). `0` (default) disables
    /// coalescing entirely → relay is spawned per producer push,
    /// byte-identical to pre-v0.5. `>0` batches relayed events.
    pub relay_coalesce_ms: u64,
    /// Max events per coalesced relay POST — the size-based flush
    /// trigger and the lost-POST blast-radius bound. Only meaningful
    /// when `relay_coalesce_ms > 0`.
    pub relay_coalesce_max_events: usize,
    /// Bounded depth of the in-process relay-coalescing queue. Full ⇒
    /// the enqueued batch is dropped + metered (never inward
    /// backpressure). Only meaningful when `relay_coalesce_ms > 0`.
    pub relay_queue_depth: usize,
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
    #[error(
        "no token-validation method configured: set WSS_MUX_OIDC_ISSUER, \
         or a handshake key (WSS_MUX_HANDSHAKE_SIGNING_KEY and/or \
         WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY[_FILE])"
    )]
    MissingHandshakeKey,
    #[error(
        "WSS_MUX_OIDC_ISSUER and a handshake key are mutually exclusive — \
         configure exactly one token-validation method"
    )]
    OidcAndHandshakeKey,
    #[error("failed to read {var} at `{path}`: {source}")]
    HandshakeKeyFile {
        var: &'static str,
        path: String,
        #[source]
        source: std::io::Error,
    },
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
    #[error("invalid {var} `{value}`: {source}")]
    InvalidUnsigned {
        var: &'static str,
        value: String,
        #[source]
        source: ParseIntError,
    },
    #[error("invalid WSS_MUX_PEERS: {0}")]
    InvalidPeers(#[source] peers::PeerUrlError),
    #[error(
        "invalid value for {var}: `{value}` (expected one of: \
         true/false, yes/no, on/off, 1/0)"
    )]
    InvalidBool { var: &'static str, value: String },
}

impl Config {
    /// Construct a `Config` programmatically from the genuinely-required
    /// fields, with every optional knob at its documented default (the
    /// same `DEFAULT_*` constants `from_env` uses). The production path
    /// is `from_env`; `new` is for embedders and for tests/benches/the
    /// load harness that build a `Config` directly — so adding a new
    /// optional field stays a one-file change here, never a hand-edited
    /// literal scattered across modules.
    pub fn new(
        push_auth_token: String,
        handshake_keys: HandshakeKeyConfig,
        manifest_path: PathBuf,
    ) -> Config {
        Config {
            listen_addr: DEFAULT_LISTEN_ADDR
                .parse()
                .expect("DEFAULT_LISTEN_ADDR is a valid socket address"),
            push_auth_token,
            handshake_keys,
            oidc: None,
            manifest_path,
            read_auth_token: None,
            queue_depth: DEFAULT_QUEUE_DEPTH,
            envelope_stream_path: DEFAULT_ENVELOPE_STREAM_PATH.to_string(),
            envelope_key_path: DEFAULT_ENVELOPE_KEY_PATH.to_string(),
            envelope_payload_path: DEFAULT_ENVELOPE_PAYLOAD_PATH.to_string(),
            inbound_rate_per_sec: DEFAULT_INBOUND_RATE,
            inbound_burst: DEFAULT_INBOUND_BURST,
            ws_publish_rate_per_sec: DEFAULT_WS_PUBLISH_RATE,
            ws_publish_burst: 0,
            ws_publish_require_sub: DEFAULT_WS_PUBLISH_REQUIRE_SUB,
            ws_publish_idle_ttl: Duration::from_secs(DEFAULT_WS_PUBLISH_IDLE_TTL_SECS),
            relay_coalesce_ms: DEFAULT_RELAY_COALESCE_MS,
            relay_coalesce_max_events: DEFAULT_RELAY_COALESCE_MAX_EVENTS,
            relay_queue_depth: DEFAULT_RELAY_QUEUE_DEPTH,
            peers: PeerConfig::default(),
        }
    }

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

        let hs256_secret = get("WSS_MUX_HANDSHAKE_SIGNING_KEY");
        let ed25519_public_pem = match get("WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY") {
            Some(pem) => Some(pem),
            None => match get("WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY_FILE") {
                Some(path) => Some(std::fs::read_to_string(&path).map_err(|source| {
                    ConfigError::HandshakeKeyFile {
                        var: "WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY_FILE",
                        path,
                        source,
                    }
                })?),
                None => None,
            },
        };
        // The handshake-key requirement is resolved jointly with OIDC
        // below (issuer XOR handshake; neither ⇒ error).
        let handshake_keys = HandshakeKeyConfig {
            hs256_secret,
            ed25519_public_pem,
        };

        let manifest_path: PathBuf = get("WSS_MUX_STREAMS_MANIFEST_PATH")
            .ok_or(ConfigError::Missing("WSS_MUX_STREAMS_MANIFEST_PATH"))?
            .into();

        let read_auth_token = get("WSS_MUX_READ_AUTH_TOKEN")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let queue_depth = match get("WSS_MUX_QUEUE_DEPTH") {
            Some(v) => {
                // `0` is the explicit "unlimited" signal (resolved in
                // the dispatcher); a literal 0-depth queue has no use.
                v.parse().map_err(|source| ConfigError::InvalidQueueDepth {
                    value: v.clone(),
                    source,
                })?
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
        let inbound_rate_per_sec = parse_u32("WSS_MUX_INBOUND_RATE", DEFAULT_INBOUND_RATE)?;
        let inbound_burst = parse_u32("WSS_MUX_INBOUND_BURST", DEFAULT_INBOUND_BURST)?;
        fn parse_bool<F>(get: &F, var: &'static str, default: bool) -> Result<bool, ConfigError>
        where
            F: Fn(&str) -> Option<String>,
        {
            match get(var) {
                Some(v) => match v.trim().to_ascii_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Ok(true),
                    "false" | "0" | "no" | "off" => Ok(false),
                    _ => Err(ConfigError::InvalidBool {
                        var,
                        value: v.clone(),
                    }),
                },
                None => Ok(default),
            }
        }

        let ws_publish_rate_per_sec =
            parse_u32("WSS_MUX_WS_PUBLISH_RATE", DEFAULT_WS_PUBLISH_RATE)?;
        // Burst defaults to 2x the steady rate when set but unconfigured,
        // matching the rest of the codebase's TokenBucket sizing.
        let ws_publish_burst = match (get("WSS_MUX_WS_PUBLISH_BURST"), ws_publish_rate_per_sec) {
            (Some(v), _) => v.parse().map_err(|source| ConfigError::InvalidUnsigned {
                var: "WSS_MUX_WS_PUBLISH_BURST",
                value: v.clone(),
                source,
            })?,
            (None, 0) => 0,
            (None, rate) => rate.saturating_mul(2),
        };
        let ws_publish_require_sub = parse_bool(
            &get,
            "WSS_MUX_WS_PUBLISH_REQUIRE_SUB",
            DEFAULT_WS_PUBLISH_REQUIRE_SUB,
        )?;
        let ws_publish_idle_ttl = Duration::from_secs(parse_u64(
            "WSS_MUX_WS_PUBLISH_IDLE_TTL_SECS",
            DEFAULT_WS_PUBLISH_IDLE_TTL_SECS,
        )?);

        let non_blank = |v: Option<String>| -> Option<String> {
            v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
        };

        // Auth method: OIDC issuer XOR handshake key(s). Both ⇒ error
        // (one trust root); neither ⇒ error (no way to validate).
        let oidc_issuer = non_blank(get("WSS_MUX_OIDC_ISSUER"));
        let has_handshake =
            handshake_keys.hs256_secret.is_some() || handshake_keys.ed25519_public_pem.is_some();
        match (oidc_issuer.is_some(), has_handshake) {
            (true, true) => return Err(ConfigError::OidcAndHandshakeKey),
            (false, false) => return Err(ConfigError::MissingHandshakeKey),
            _ => {}
        }
        let oidc = match oidc_issuer {
            Some(issuer) => Some(OidcConfig {
                issuer,
                // Pinning an audience is mandatory — accepting any
                // `aud` would let tokens minted for another relying
                // party authenticate here.
                audience: non_blank(get("WSS_MUX_OIDC_AUDIENCE"))
                    .ok_or(ConfigError::Missing("WSS_MUX_OIDC_AUDIENCE"))?,
                discovery_url: non_blank(get("WSS_MUX_OIDC_DISCOVERY_URL")),
                jwks_url: non_blank(get("WSS_MUX_OIDC_JWKS_URL")),
                groups_claim: non_blank(get("WSS_MUX_OIDC_GROUPS_CLAIM"))
                    .unwrap_or_else(|| DEFAULT_OIDC_GROUPS_CLAIM.to_string()),
                principal_prefix: non_blank(get("WSS_MUX_OIDC_PRINCIPAL_PREFIX"))
                    .unwrap_or_default(),
                jwks_refresh: Duration::from_secs(parse_u64(
                    "WSS_MUX_OIDC_JWKS_REFRESH",
                    DEFAULT_OIDC_JWKS_REFRESH_SECS,
                )?),
            }),
            None => None,
        };

        let parse_usize = |var: &'static str, default: usize| -> Result<usize, ConfigError> {
            match get(var) {
                Some(v) => v.parse().map_err(|source| ConfigError::InvalidUnsigned {
                    var,
                    value: v.clone(),
                    source,
                }),
                None => Ok(default),
            }
        };
        let relay_coalesce_ms = parse_u64("WSS_MUX_RELAY_COALESCE_MS", DEFAULT_RELAY_COALESCE_MS)?;
        let relay_coalesce_max_events = parse_usize(
            "WSS_MUX_RELAY_COALESCE_MAX_EVENTS",
            DEFAULT_RELAY_COALESCE_MAX_EVENTS,
        )?;
        let relay_queue_depth =
            parse_usize("WSS_MUX_RELAY_QUEUE_DEPTH", DEFAULT_RELAY_QUEUE_DEPTH)?;

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
            handshake_keys,
            oidc,
            manifest_path,
            read_auth_token,
            queue_depth,
            envelope_stream_path,
            envelope_key_path,
            envelope_payload_path,
            inbound_rate_per_sec,
            inbound_burst,
            ws_publish_rate_per_sec,
            ws_publish_burst,
            ws_publish_require_sub,
            ws_publish_idle_ttl,
            relay_coalesce_ms,
            relay_coalesce_max_events,
            relay_queue_depth,
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
        assert_eq!(
            cfg.handshake_keys.hs256_secret.as_deref(),
            Some("handshake-secret")
        );
        assert!(cfg.handshake_keys.ed25519_public_pem.is_none());
        assert!(
            cfg.oidc.is_none(),
            "OIDC is off unless WSS_MUX_OIDC_ISSUER set"
        );
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
    fn ws_publish_rate_defaults_to_disabled() {
        let cfg = Config::from_getter(env(&minimal())).expect("config");
        assert_eq!(cfg.ws_publish_rate_per_sec, 0);
        assert_eq!(cfg.ws_publish_burst, 0);
        assert!(!cfg.ws_publish_require_sub);
        assert_eq!(
            cfg.ws_publish_idle_ttl,
            Duration::from_secs(DEFAULT_WS_PUBLISH_IDLE_TTL_SECS)
        );
    }

    #[test]
    fn ws_publish_burst_defaults_to_2x_rate_when_unset() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_WS_PUBLISH_RATE", "10"));
        // No explicit burst — default should be 2 * rate.
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.ws_publish_rate_per_sec, 10);
        assert_eq!(cfg.ws_publish_burst, 20);
    }

    #[test]
    fn ws_publish_explicit_burst_overrides_default() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_WS_PUBLISH_RATE", "10"));
        pairs.push(("WSS_MUX_WS_PUBLISH_BURST", "100"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.ws_publish_rate_per_sec, 10);
        assert_eq!(cfg.ws_publish_burst, 100);
    }

    #[test]
    fn ws_publish_require_sub_parses_common_truthy_values() {
        for v in ["true", "TRUE", "1", "yes", "on", " true "] {
            let mut pairs = minimal();
            pairs.push(("WSS_MUX_WS_PUBLISH_REQUIRE_SUB", v));
            let cfg = Config::from_getter(env(&pairs)).expect("config");
            assert!(cfg.ws_publish_require_sub, "value `{v}` should parse true");
        }
        for v in ["false", "FALSE", "0", "no", "off"] {
            let mut pairs = minimal();
            pairs.push(("WSS_MUX_WS_PUBLISH_REQUIRE_SUB", v));
            let cfg = Config::from_getter(env(&pairs)).expect("config");
            assert!(
                !cfg.ws_publish_require_sub,
                "value `{v}` should parse false"
            );
        }
    }

    #[test]
    fn ws_publish_require_sub_rejects_garbage() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_WS_PUBLISH_REQUIRE_SUB", "maybe"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::InvalidBool { var, .. }
            if var == "WSS_MUX_WS_PUBLISH_REQUIRE_SUB"));
    }

    #[test]
    fn ws_publish_idle_ttl_override() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_WS_PUBLISH_IDLE_TTL_SECS", "60"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.ws_publish_idle_ttl, Duration::from_secs(60));
    }

    #[test]
    fn read_auth_token_unset_by_default() {
        let cfg = Config::from_getter(env(&minimal())).expect("config");
        assert!(cfg.read_auth_token.is_none());
    }

    #[test]
    fn read_auth_token_set_from_env() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_READ_AUTH_TOKEN", "read-secret"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.read_auth_token.as_deref(), Some("read-secret"));
    }

    #[test]
    fn read_auth_token_blank_is_treated_as_unset() {
        // A whitespace-only value is the same as missing — protects
        // operators against a quoted-empty env-var that would
        // otherwise admit a `Bearer ` (empty token) request.
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_READ_AUTH_TOKEN", "   "));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert!(cfg.read_auth_token.is_none());
    }

    #[test]
    fn relay_coalesce_defaults_and_overrides() {
        let cfg = Config::from_getter(env(&minimal())).expect("config");
        assert_eq!(cfg.relay_coalesce_ms, 0);
        assert_eq!(
            cfg.relay_coalesce_max_events,
            DEFAULT_RELAY_COALESCE_MAX_EVENTS
        );
        assert_eq!(cfg.relay_queue_depth, DEFAULT_RELAY_QUEUE_DEPTH);

        let mut pairs = minimal();
        pairs.push(("WSS_MUX_RELAY_COALESCE_MS", "25"));
        pairs.push(("WSS_MUX_RELAY_COALESCE_MAX_EVENTS", "512"));
        pairs.push(("WSS_MUX_RELAY_QUEUE_DEPTH", "4096"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.relay_coalesce_ms, 25);
        assert_eq!(cfg.relay_coalesce_max_events, 512);
        assert_eq!(cfg.relay_queue_depth, 4096);
    }

    #[test]
    fn invalid_relay_coalesce_ms_is_error() {
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_RELAY_COALESCE_MS", "soon"));
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(
            err,
            ConfigError::InvalidUnsigned {
                var: "WSS_MUX_RELAY_COALESCE_MS",
                ..
            }
        ));
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
    fn missing_all_handshake_keys_is_error() {
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
        ];
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::MissingHandshakeKey));
    }

    #[test]
    fn ed25519_only_satisfies_handshake_key_requirement() {
        // No HS256 secret; an inline Ed25519 PEM alone is sufficient.
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
            (
                "WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY",
                "-----BEGIN PUBLIC KEY-----\nABC\n-----END PUBLIC KEY-----\n",
            ),
        ];
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert!(cfg.handshake_keys.hs256_secret.is_none());
        assert!(cfg
            .handshake_keys
            .ed25519_public_pem
            .as_deref()
            .unwrap()
            .contains("BEGIN PUBLIC KEY"));
    }

    #[test]
    fn oidc_issuer_with_a_handshake_key_is_mutually_exclusive_error() {
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
            ("WSS_MUX_HANDSHAKE_SIGNING_KEY", "hs"),
            ("WSS_MUX_OIDC_ISSUER", "https://idp.example"),
            ("WSS_MUX_OIDC_AUDIENCE", "wss-mux"),
        ];
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::OidcAndHandshakeKey));
    }

    #[test]
    fn oidc_issuer_alone_configures_oidc_with_defaults() {
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
            ("WSS_MUX_OIDC_ISSUER", "https://idp.example"),
            ("WSS_MUX_OIDC_AUDIENCE", "wss-mux"),
        ];
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        // OIDC replaces the handshake-key requirement entirely.
        assert!(cfg.handshake_keys.hs256_secret.is_none());
        assert!(cfg.handshake_keys.ed25519_public_pem.is_none());
        let oidc = cfg.oidc.expect("oidc configured");
        assert_eq!(oidc.issuer, "https://idp.example");
        assert_eq!(oidc.audience, "wss-mux");
        assert!(oidc.jwks_url.is_none(), "discovery unless overridden");
        assert!(
            oidc.discovery_url.is_none(),
            "discovery base defaults to issuer unless overridden"
        );
        assert_eq!(oidc.groups_claim, "groups");
        assert_eq!(oidc.principal_prefix, "");
        assert_eq!(oidc.jwks_refresh, std::time::Duration::from_secs(300));
    }

    #[test]
    fn oidc_discovery_url_is_read_independently_of_issuer() {
        // BACKCHANNEL_DYNAMIC scenario: tokens carry the public issuer
        // (what `iss` claims), discovery is fetched in-cluster.
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
            ("WSS_MUX_OIDC_ISSUER", "https://auth.example.com/realms/x"),
            ("WSS_MUX_OIDC_AUDIENCE", "wss-mux"),
            (
                "WSS_MUX_OIDC_DISCOVERY_URL",
                "http://keycloak:8080/realms/x",
            ),
        ];
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        let oidc = cfg.oidc.expect("oidc");
        assert_eq!(oidc.issuer, "https://auth.example.com/realms/x");
        assert_eq!(
            oidc.discovery_url.as_deref(),
            Some("http://keycloak:8080/realms/x")
        );
        assert!(
            oidc.jwks_url.is_none(),
            "jwks_url unset — discovery will be reached via discovery_url"
        );
    }

    #[test]
    fn oidc_issuer_without_audience_is_error() {
        // Pinning an audience is mandatory.
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
            ("WSS_MUX_OIDC_ISSUER", "https://idp.example"),
        ];
        let err = Config::from_getter(env(&pairs)).expect_err("error");
        assert!(matches!(err, ConfigError::Missing("WSS_MUX_OIDC_AUDIENCE")));
    }

    #[test]
    fn oidc_overrides_parse_and_bad_refresh_errors() {
        let base = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
            ("WSS_MUX_OIDC_ISSUER", "https://idp.example"),
            ("WSS_MUX_OIDC_AUDIENCE", "wss-mux"),
            ("WSS_MUX_OIDC_JWKS_URL", "https://idp.example/keys"),
            ("WSS_MUX_OIDC_GROUPS_CLAIM", "roles"),
            ("WSS_MUX_OIDC_PRINCIPAL_PREFIX", "role:"),
            ("WSS_MUX_OIDC_JWKS_REFRESH", "60"),
        ];
        let cfg = Config::from_getter(env(&base)).expect("config");
        let oidc = cfg.oidc.expect("oidc");
        assert_eq!(oidc.jwks_url.as_deref(), Some("https://idp.example/keys"));
        assert_eq!(oidc.groups_claim, "roles");
        assert_eq!(oidc.principal_prefix, "role:");
        assert_eq!(oidc.jwks_refresh, std::time::Duration::from_secs(60));

        let mut bad = base.clone();
        bad.retain(|(k, _)| *k != "WSS_MUX_OIDC_JWKS_REFRESH");
        bad.push(("WSS_MUX_OIDC_JWKS_REFRESH", "nope"));
        assert!(matches!(
            Config::from_getter(env(&bad)).expect_err("error"),
            ConfigError::InvalidUnsigned {
                var: "WSS_MUX_OIDC_JWKS_REFRESH",
                ..
            }
        ));
    }

    #[test]
    fn ed25519_file_is_read_into_pem() {
        let path = std::env::temp_dir().join(format!(
            "wss-mux-ed25519-{}-{}.pem",
            std::process::id(),
            std::thread::current().name().unwrap_or("t")
        ));
        std::fs::write(
            &path,
            "-----BEGIN PUBLIC KEY-----\nFROMFILE\n-----END PUBLIC KEY-----\n",
        )
        .expect("write key");
        let p = path.to_string_lossy().to_string();
        let pairs = vec![
            ("WSS_MUX_PUSH_AUTH_TOKEN", "t"),
            ("WSS_MUX_STREAMS_MANIFEST_PATH", "p"),
            ("WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY_FILE", p.as_str()),
        ];
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert!(cfg
            .handshake_keys
            .ed25519_public_pem
            .as_deref()
            .unwrap()
            .contains("FROMFILE"));
        let _ = std::fs::remove_file(&path);
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
    fn zero_queue_depth_means_unlimited() {
        // `0` is the explicit "unlimited" signal (a 0-depth queue would
        // be useless as a literal), so it parses rather than erroring.
        let mut pairs = minimal();
        pairs.push(("WSS_MUX_QUEUE_DEPTH", "0"));
        let cfg = Config::from_getter(env(&pairs)).expect("config");
        assert_eq!(cfg.queue_depth, 0);
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
