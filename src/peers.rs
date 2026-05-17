//! Peer discovery for cross-instance event relay.
//!
//! An instance that receives a producer push relays it once to its
//! discovered peers; every instance fans out to its own clients. This
//! module owns *discovery* — turning configuration into a concrete set
//! of peer base URLs — plus the relay HTTP client builder. It does
//! **not** perform any relay itself (that wiring lands later) and is
//! intentionally free of `AppState` coupling so every piece is unit
//! testable in isolation.
//!
//! Sources, in priority order:
//! 1. `WSS_MUX_PEER_RELAY=off` — relay disabled entirely.
//! 2. static `WSS_MUX_PEERS` CSV — a fixed fleet, no DNS polling.
//! 3. `WSS_MUX_PEER_DNS` — an explicit FQDN, resolved on the refresh
//!    interval.
//! 4. composed `{service}.{namespace}.svc.{cluster_domain}` — the
//!    zero-config conventional k8s headless Service.
//!
//! Resolving nothing yields an empty peer set ⇒ behaviour is
//! byte-identical to a single instance.

use std::net::{IpAddr, SocketAddr};

use anyhow::Context;
use thiserror::Error;

use crate::config::PeerConfig;

/// In-cluster path k8s populates with the pod's namespace. Present with
/// zero configuration whenever a ServiceAccount is mounted (the
/// default).
pub const SERVICEACCOUNT_NAMESPACE_PATH: &str =
    "/var/run/secrets/kubernetes.io/serviceaccount/namespace";

/// Default headless Service name. The operations runbook ships a
/// `Service` named exactly this so the conventional deploy needs zero
/// peer configuration.
pub const DEFAULT_PEER_SERVICE: &str = "wss-mux-headless";

/// Default k8s cluster DNS domain.
pub const DEFAULT_CLUSTER_DOMAIN: &str = "cluster.local";

/// Default per-relay request timeout. In-cluster pod-to-pod RTT is
/// single-digit ms; 500ms is wide headroom for GC/scheduling jitter
/// while still cutting a degraded peer loose quickly.
pub const DEFAULT_RELAY_TIMEOUT_MS: u64 = 500;

/// Default DNS re-resolution interval. k8s propagates pod readiness in
/// ~1-2s and CoreDNS answers are cached/cheap, so a tight window beats
/// a long scale-up/down blind spot.
pub const DEFAULT_DNS_REFRESH_SECS: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

impl Scheme {
    fn as_str(self) -> &'static str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
        }
    }
}

/// A resolved peer's base URL. The relay path is appended by the caller
/// (`{base}/internal/v1/relay`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerUrl {
    pub scheme: Scheme,
    pub host: String,
    pub port: u16,
}

impl PeerUrl {
    /// `scheme://host:port`, bracketing IPv6 literals.
    pub fn base(&self) -> String {
        if self.host.contains(':') && !self.host.starts_with('[') {
            format!("{}://[{}]:{}", self.scheme.as_str(), self.host, self.port)
        } else {
            format!("{}://{}:{}", self.scheme.as_str(), self.host, self.port)
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PeerUrlError {
    #[error("empty peer entry")]
    Empty,
    #[error("unsupported scheme in peer `{0}` (expected http or https)")]
    UnsupportedScheme(String),
    #[error("missing `:port` in peer `{0}`")]
    MissingPort(String),
    #[error("invalid port in peer `{0}`")]
    InvalidPort(String),
    #[error("missing host in peer `{0}`")]
    MissingHost(String),
    #[error("unexpected path in peer `{0}` (base URL only)")]
    UnexpectedPath(String),
}

/// Parse one `[http://|https://]host:port` entry. No scheme defaults to
/// `http` (the in-cluster plaintext norm). A port is mandatory — peers
/// are explicit addresses, not browser URLs. An optional trailing `/`
/// is tolerated; any other path is rejected.
pub fn parse_peer_url(raw: &str) -> Result<PeerUrl, PeerUrlError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PeerUrlError::Empty);
    }

    let (scheme, rest) = match trimmed.split_once("://") {
        Some((s, r)) => match s.to_ascii_lowercase().as_str() {
            "http" => (Scheme::Http, r),
            "https" => (Scheme::Https, r),
            _ => return Err(PeerUrlError::UnsupportedScheme(raw.to_string())),
        },
        None => (Scheme::Http, trimmed),
    };

    let (authority, path) = match rest.split_once('/') {
        Some((a, p)) => (a, Some(p)),
        None => (rest, None),
    };
    if let Some(p) = path {
        if !p.is_empty() {
            return Err(PeerUrlError::UnexpectedPath(raw.to_string()));
        }
    }
    if authority.is_empty() {
        return Err(PeerUrlError::MissingHost(raw.to_string()));
    }

    let (host, port_str) = if let Some(stripped) = authority.strip_prefix('[') {
        // IPv6 literal: `[addr]:port`.
        let (host, after) = stripped
            .split_once(']')
            .ok_or_else(|| PeerUrlError::MissingHost(raw.to_string()))?;
        let port_str = after
            .strip_prefix(':')
            .ok_or_else(|| PeerUrlError::MissingPort(raw.to_string()))?;
        (host, port_str)
    } else {
        let (host, port_str) = authority
            .rsplit_once(':')
            .ok_or_else(|| PeerUrlError::MissingPort(raw.to_string()))?;
        (host, port_str)
    };

    if host.is_empty() {
        return Err(PeerUrlError::MissingHost(raw.to_string()));
    }
    let port: u16 = port_str
        .parse()
        .map_err(|_| PeerUrlError::InvalidPort(raw.to_string()))?;

    Ok(PeerUrl {
        scheme,
        host: host.to_string(),
        port,
    })
}

/// Parse a comma-separated list of peer entries (`WSS_MUX_PEERS`).
/// Whitespace around entries is trimmed; empty entries are skipped.
pub fn parse_static_peers(csv: &str) -> Result<Vec<PeerUrl>, PeerUrlError> {
    csv.split(',')
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(parse_peer_url)
        .collect()
}

/// Compose the conventional headless-Service FQDN.
pub fn compose_fqdn(service: &str, namespace: &str, cluster_domain: &str) -> String {
    format!("{service}.{namespace}.svc.{cluster_domain}")
}

/// Resolve the effective namespace: an explicit override wins;
/// otherwise the ServiceAccount file is consulted via `read_file`
/// (injected for testability). Returns `None` when neither yields a
/// non-empty value — discovery then has no composed FQDN to resolve.
pub fn resolve_namespace<R>(explicit: Option<&str>, read_file: R) -> Option<String>
where
    R: FnOnce() -> Option<String>,
{
    let non_blank = |s: String| {
        let t = s.trim().to_string();
        (!t.is_empty()).then_some(t)
    };
    explicit
        .map(str::to_string)
        .and_then(non_blank)
        .or_else(|| read_file().and_then(non_blank))
}

/// Drop any resolved address that is this host. `probe_source` returns
/// the source IP the OS would select to reach a given address (the
/// UDP-connect trick); when that equals the destination IP, the
/// destination is one of our own interface addresses. Order is
/// preserved and duplicates are removed.
pub fn exclude_self<F>(resolved: Vec<SocketAddr>, probe_source: F) -> Vec<SocketAddr>
where
    F: Fn(IpAddr) -> Option<IpAddr>,
{
    let mut seen = std::collections::HashSet::new();
    resolved
        .into_iter()
        .filter(|addr| seen.insert(*addr))
        .filter(|addr| probe_source(addr.ip()) != Some(addr.ip()))
        .collect()
}

/// Resolve an FQDN to its current address set via the cluster's DNS
/// (`tokio::net::lookup_host`). A failed lookup (NXDOMAIN, no records)
/// yields an empty set — the inert, single-instance-equivalent case.
pub async fn resolve_fqdn(fqdn: &str, port: u16) -> Vec<SocketAddr> {
    match tokio::net::lookup_host((fqdn, port)).await {
        Ok(addrs) => addrs.collect(),
        Err(_) => Vec::new(),
    }
}

/// Build the relay HTTP client from peer config: per-relay timeout,
/// optional private-CA trust, optional client-cert mutual TLS. Plain
/// `http://` peers need none of the TLS knobs and are the default.
pub fn build_relay_client(cfg: &PeerConfig) -> anyhow::Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder().timeout(cfg.relay_timeout);

    if let Some(ca) = &cfg.ca_file {
        let pem = std::fs::read(ca)
            .with_context(|| format!("reading WSS_MUX_PEER_CA_FILE `{}`", ca.display()))?;
        let cert = reqwest::Certificate::from_pem(&pem)
            .with_context(|| format!("parsing WSS_MUX_PEER_CA_FILE `{}`", ca.display()))?;
        builder = builder.add_root_certificate(cert);
    }

    match (&cfg.client_cert, &cfg.client_key) {
        (Some(cert_path), Some(key_path)) => {
            let mut pem = std::fs::read(cert_path).with_context(|| {
                format!("reading WSS_MUX_PEER_CLIENT_CERT `{}`", cert_path.display())
            })?;
            pem.push(b'\n');
            pem.extend(std::fs::read(key_path).with_context(|| {
                format!("reading WSS_MUX_PEER_CLIENT_KEY `{}`", key_path.display())
            })?);
            let identity = reqwest::Identity::from_pem(&pem)
                .context("building mutual-TLS identity from client cert + key")?;
            builder = builder.identity(identity);
        }
        (None, None) => {}
        _ => anyhow::bail!(
            "WSS_MUX_PEER_CLIENT_CERT and WSS_MUX_PEER_CLIENT_KEY must both be set for mutual TLS"
        ),
    }

    builder.build().context("building relay HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Wave A: peer-URL parsing -------------------------------

    #[test]
    fn bare_host_port_defaults_to_http() {
        let p = parse_peer_url("10.0.0.5:8080").expect("parse");
        assert_eq!(
            p,
            PeerUrl {
                scheme: Scheme::Http,
                host: "10.0.0.5".into(),
                port: 8080
            }
        );
        assert_eq!(p.base(), "http://10.0.0.5:8080");
    }

    #[test]
    fn explicit_http_and_https_schemes_parse() {
        let h = parse_peer_url("http://a.example:9000").expect("http");
        assert_eq!(h.scheme, Scheme::Http);
        assert_eq!(h.host, "a.example");
        assert_eq!(h.port, 9000);

        let s = parse_peer_url("https://b.example:443").expect("https");
        assert_eq!(s.scheme, Scheme::Https);
        assert_eq!(s.base(), "https://b.example:443");
    }

    #[test]
    fn ipv6_literal_is_bracketed_in_base() {
        let p = parse_peer_url("http://[2001:db8::1]:8080").expect("parse");
        assert_eq!(p.scheme, Scheme::Http);
        assert_eq!(p.host, "2001:db8::1");
        assert_eq!(p.port, 8080);
        assert_eq!(p.base(), "http://[2001:db8::1]:8080");
    }

    #[test]
    fn trailing_slash_is_tolerated() {
        let p = parse_peer_url("http://a.example:8080/").expect("parse");
        assert_eq!(p.host, "a.example");
        assert_eq!(p.port, 8080);
    }

    #[test]
    fn unsupported_scheme_is_rejected() {
        let e = parse_peer_url("ws://a.example:8080").expect_err("reject");
        assert!(matches!(e, PeerUrlError::UnsupportedScheme(_)));
    }

    #[test]
    fn missing_port_is_rejected() {
        let e = parse_peer_url("http://a.example").expect_err("reject");
        assert!(matches!(e, PeerUrlError::MissingPort(_)));
    }

    #[test]
    fn non_numeric_port_is_rejected() {
        let e = parse_peer_url("a.example:http").expect_err("reject");
        assert!(matches!(e, PeerUrlError::InvalidPort(_)));
    }

    #[test]
    fn empty_entry_is_rejected() {
        let e = parse_peer_url("   ").expect_err("reject");
        assert!(matches!(e, PeerUrlError::Empty));
    }

    #[test]
    fn path_other_than_trailing_slash_is_rejected() {
        let e = parse_peer_url("http://a.example:8080/relay").expect_err("reject");
        assert!(matches!(e, PeerUrlError::UnexpectedPath(_)));
    }

    #[test]
    fn static_csv_parses_trims_and_skips_blanks() {
        let v = parse_static_peers(" http://a:8080 , b:9090 ,,https://c:443 ").expect("parse");
        assert_eq!(
            v,
            vec![
                PeerUrl {
                    scheme: Scheme::Http,
                    host: "a".into(),
                    port: 8080
                },
                PeerUrl {
                    scheme: Scheme::Http,
                    host: "b".into(),
                    port: 9090
                },
                PeerUrl {
                    scheme: Scheme::Https,
                    host: "c".into(),
                    port: 443
                },
            ]
        );
    }

    #[test]
    fn static_csv_propagates_entry_error() {
        let e = parse_static_peers("a:8080,bad:port").expect_err("reject");
        assert!(matches!(e, PeerUrlError::InvalidPort(_)));
    }

    #[test]
    fn static_csv_all_blank_is_empty() {
        assert_eq!(parse_static_peers("  , ,").expect("parse"), vec![]);
    }

    // ---- Wave B: FQDN + namespace -------------------------------

    #[test]
    fn compose_fqdn_uses_conventional_shape() {
        assert_eq!(
            compose_fqdn("wss-mux-headless", "prod", "cluster.local"),
            "wss-mux-headless.prod.svc.cluster.local"
        );
        assert_eq!(
            compose_fqdn(DEFAULT_PEER_SERVICE, "default", DEFAULT_CLUSTER_DOMAIN),
            "wss-mux-headless.default.svc.cluster.local"
        );
    }

    #[test]
    fn resolve_namespace_prefers_explicit_override() {
        let ns = resolve_namespace(Some("override-ns"), || {
            panic!("file must not be read when override is present")
        });
        assert_eq!(ns.as_deref(), Some("override-ns"));
    }

    #[test]
    fn resolve_namespace_reads_serviceaccount_file_when_no_override() {
        let ns = resolve_namespace(None, || Some("file-ns".to_string()));
        assert_eq!(ns.as_deref(), Some("file-ns"));
    }

    #[test]
    fn resolve_namespace_trims_file_content() {
        let ns = resolve_namespace(None, || Some("  team-a\n".to_string()));
        assert_eq!(ns.as_deref(), Some("team-a"));
    }

    #[test]
    fn resolve_namespace_blank_override_falls_back_to_file() {
        let ns = resolve_namespace(Some("   "), || Some("file-ns".to_string()));
        assert_eq!(ns.as_deref(), Some("file-ns"));
    }

    #[test]
    fn resolve_namespace_blank_file_yields_none() {
        assert_eq!(resolve_namespace(None, || Some("\n".to_string())), None);
    }

    #[test]
    fn resolve_namespace_none_when_neither_present() {
        assert_eq!(resolve_namespace(None, || None), None);
    }

    // ---- Wave C: self-exclusion ---------------------------------

    fn sa(s: &str) -> SocketAddr {
        s.parse().expect("socket addr")
    }

    #[test]
    fn exclude_self_drops_addresses_that_are_this_host() {
        let self_ip: IpAddr = "10.0.0.9".parse().unwrap();
        let resolved = vec![
            sa("10.0.0.1:8080"),
            sa("10.0.0.9:8080"),
            sa("10.0.0.2:8080"),
        ];
        let kept = exclude_self(resolved, |ip| {
            if ip == self_ip {
                Some(self_ip) // OS would source-from this IP ⇒ it's us
            } else {
                Some("10.0.0.250".parse().unwrap()) // a different source IP
            }
        });
        assert_eq!(kept, vec![sa("10.0.0.1:8080"), sa("10.0.0.2:8080")]);
    }

    #[test]
    fn exclude_self_preserves_order_and_dedupes() {
        let resolved = vec![
            sa("10.0.0.1:8080"),
            sa("10.0.0.2:8080"),
            sa("10.0.0.1:8080"),
            sa("10.0.0.3:8080"),
        ];
        let kept = exclude_self(resolved, |_| Some("203.0.113.7".parse().unwrap()));
        assert_eq!(
            kept,
            vec![
                sa("10.0.0.1:8080"),
                sa("10.0.0.2:8080"),
                sa("10.0.0.3:8080")
            ]
        );
    }

    #[test]
    fn exclude_self_keeps_address_when_probe_fails() {
        // Can't prove it's us ⇒ keep it. The one-hop path guard makes a
        // mistaken self-relay harmless; dropping a real peer would not be.
        let resolved = vec![sa("10.0.0.5:8080")];
        let kept = exclude_self(resolved, |_| None);
        assert_eq!(kept, vec![sa("10.0.0.5:8080")]);
    }

    // ---- Wave D: DNS resolution + relay client ------------------

    use crate::config::PeerConfig;

    #[tokio::test]
    async fn resolve_fqdn_localhost_yields_loopback() {
        let addrs = resolve_fqdn("localhost", 8080).await;
        assert!(!addrs.is_empty(), "localhost must resolve");
        for a in &addrs {
            assert!(a.ip().is_loopback(), "{a} should be loopback");
            assert_eq!(a.port(), 8080);
        }
    }

    #[tokio::test]
    async fn resolve_fqdn_unresolvable_yields_empty() {
        // `.invalid` is reserved (RFC 6761) ⇒ guaranteed NXDOMAIN.
        let addrs = resolve_fqdn("wss-mux-no-such-host.invalid", 8080).await;
        assert!(addrs.is_empty());
    }

    #[test]
    fn relay_client_builds_with_defaults() {
        // Plain http:// peers: no TLS knobs, must build cleanly.
        build_relay_client(&PeerConfig::default()).expect("default client builds");
    }

    #[test]
    fn relay_client_errors_on_missing_ca_file() {
        let cfg = PeerConfig {
            ca_file: Some("/nonexistent/wss-mux-ca.pem".into()),
            ..PeerConfig::default()
        };
        assert!(build_relay_client(&cfg).is_err());
    }

    #[test]
    fn relay_client_errors_on_half_configured_mtls() {
        let cfg = PeerConfig {
            client_cert: Some("/some/cert.pem".into()),
            client_key: None,
            ..PeerConfig::default()
        };
        let err = build_relay_client(&cfg).expect_err("half mTLS must error");
        assert!(err.to_string().contains("must both be set"));
    }
}
