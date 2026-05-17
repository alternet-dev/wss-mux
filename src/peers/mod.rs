//! Peer discovery for cross-instance event relay.
//!
//! An instance that receives a producer push relays it once to its
//! discovered peers; every instance fans out to its own clients. This
//! module owns *discovery* — turning configuration into a concrete set
//! of peer base URLs — plus the relay HTTP client builder. It does
//! **not** perform the relay itself; that lives in the server layer
//! (`server::http`). Every piece here is free of `AppState` coupling so
//! it is unit testable in isolation.
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
//!
//! Layout: `url` (parsing + `PeerUrl`), `discovery` (FQDN / namespace /
//! DNS / self-exclusion), `client` (relay HTTP client). The
//! orchestration that ties them together (`discover`) lives here.

mod client;
mod discovery;
mod url;

pub use client::build_relay_client;
pub use discovery::{
    compose_fqdn, exclude_self, os_source_ip, resolve_fqdn, resolve_namespace,
    SERVICEACCOUNT_NAMESPACE_PATH,
};
pub use url::{parse_peer_url, parse_static_peers, PeerUrl, PeerUrlError, Scheme};

use std::net::IpAddr;

use crate::config::PeerConfig;

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

/// Resolve the live peer set from configuration. Precedence: relay-off
/// ⇒ none; static list ⇒ as-is (no DNS); explicit `peer_dns` ⇒ resolve;
/// otherwise compose the conventional headless-Service FQDN from the
/// resolved namespace and resolve that. DNS-discovered peers are plain
/// `http` (the in-cluster norm). Self is dropped via `probe`. `read_ns`
/// and `probe` are injected for testability.
pub async fn discover_with<R, P>(
    cfg: &PeerConfig,
    self_port: u16,
    read_ns: R,
    probe: P,
) -> Vec<PeerUrl>
where
    R: FnOnce() -> Option<String>,
    P: Fn(IpAddr) -> Option<IpAddr>,
{
    if !cfg.relay_enabled {
        return Vec::new();
    }
    if !cfg.static_peers.is_empty() {
        return cfg.static_peers.clone();
    }
    let fqdn = match &cfg.peer_dns {
        Some(dns) => dns.clone(),
        None => match resolve_namespace(cfg.namespace_override.as_deref(), read_ns) {
            Some(ns) => compose_fqdn(&cfg.service, &ns, &cfg.cluster_domain),
            None => return Vec::new(),
        },
    };
    let resolved = resolve_fqdn(&fqdn, self_port).await;
    exclude_self(resolved, probe)
        .into_iter()
        .map(|sa| PeerUrl {
            scheme: Scheme::Http,
            host: sa.ip().to_string(),
            port: sa.port(),
        })
        .collect()
}

/// Production discovery: real ServiceAccount-file namespace read and
/// real OS source-IP probe.
pub async fn discover(cfg: &PeerConfig, self_port: u16) -> Vec<PeerUrl> {
    discover_with(
        cfg,
        self_port,
        || std::fs::read_to_string(SERVICEACCOUNT_NAMESPACE_PATH).ok(),
        os_source_ip,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PeerConfig;

    #[tokio::test]
    async fn discover_relay_off_is_empty() {
        let cfg = PeerConfig {
            relay_enabled: false,
            static_peers: vec![PeerUrl {
                scheme: Scheme::Http,
                host: "a".into(),
                port: 1,
            }],
            ..PeerConfig::default()
        };
        let peers = discover_with(
            &cfg,
            8080,
            || panic!("ns read must not happen when relay is off"),
            |_| panic!("probe must not happen when relay is off"),
        )
        .await;
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn discover_static_peers_bypass_dns() {
        let statics = vec![
            PeerUrl {
                scheme: Scheme::Http,
                host: "10.0.0.1".into(),
                port: 8080,
            },
            PeerUrl {
                scheme: Scheme::Https,
                host: "10.0.0.2".into(),
                port: 9090,
            },
        ];
        let cfg = PeerConfig {
            static_peers: statics.clone(),
            ..PeerConfig::default()
        };
        let peers = discover_with(
            &cfg,
            8080,
            || panic!("ns read must not happen for static peers"),
            |_| panic!("probe must not happen for static peers"),
        )
        .await;
        assert_eq!(peers, statics);
    }

    #[tokio::test]
    async fn discover_no_namespace_is_inert() {
        // No static, no peer_dns, no namespace override, file read fails
        // ⇒ nothing to compose ⇒ empty (single-instance-equivalent).
        let cfg = PeerConfig::default();
        let peers = discover_with(&cfg, 8080, || None, |_| None).await;
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn discover_peer_dns_resolves_and_excludes_self() {
        // localhost resolves to loopback; the real probe reports loopback
        // as self ⇒ every resolved address is excluded ⇒ empty.
        let cfg = PeerConfig {
            peer_dns: Some("localhost".into()),
            ..PeerConfig::default()
        };
        let peers = discover_with(
            &cfg,
            8080,
            || panic!("ns read must not happen when peer_dns is set"),
            os_source_ip,
        )
        .await;
        assert!(peers.is_empty(), "loopback peers must be self-excluded");
    }

    #[tokio::test]
    async fn discover_composes_fqdn_from_namespace_and_is_inert_on_nxdomain() {
        // namespace present ⇒ composed FQDN is resolved. Pin the domain to
        // the RFC 6761 reserved `.invalid` TLD so the lookup is a fast,
        // deterministic NXDOMAIN (a real `.cluster.local` name triggers
        // slow resolver search retries). Empty result proves the compose
        // path is taken without error.
        let cfg = PeerConfig {
            namespace_override: Some("team-a".into()),
            cluster_domain: "invalid".into(),
            ..PeerConfig::default()
        };
        let peers = discover_with(
            &cfg,
            8080,
            || None,
            |_| Some("203.0.113.1".parse().unwrap()),
        )
        .await;
        assert!(peers.is_empty());
    }
}
