//! Turning configuration into concrete addresses: namespace + FQDN
//! composition, DNS resolution, and self-exclusion (incl. the
//! dependency-free OS source-IP probe).

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// In-cluster path k8s populates with the pod's namespace. Present with
/// zero configuration whenever a ServiceAccount is mounted (the
/// default).
pub const SERVICEACCOUNT_NAMESPACE_PATH: &str =
    "/var/run/secrets/kubernetes.io/serviceaccount/namespace";

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

/// Source IP the OS would select to reach `dst` (UDP-connect trick — no
/// packets are sent; `connect` on a datagram socket only fixes the peer
/// and lets the kernel pick a source). When this equals `dst`, `dst` is
/// one of our own interface addresses: the basis for self-exclusion.
pub fn os_source_ip(dst: IpAddr) -> Option<IpAddr> {
    let bind: SocketAddr = if dst.is_ipv4() {
        (Ipv4Addr::UNSPECIFIED, 0).into()
    } else {
        (Ipv6Addr::UNSPECIFIED, 0).into()
    };
    let sock = std::net::UdpSocket::bind(bind).ok()?;
    sock.connect(SocketAddr::new(dst, 9)).ok()?;
    sock.local_addr().ok().map(|a| a.ip())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peers::{DEFAULT_CLUSTER_DOMAIN, DEFAULT_PEER_SERVICE};

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
    fn os_source_ip_loopback_is_itself() {
        // Connecting a UDP socket to loopback makes the kernel select
        // the loopback address as source ⇒ proves the self-detect basis.
        let lo: IpAddr = "127.0.0.1".parse().unwrap();
        assert_eq!(os_source_ip(lo), Some(lo));
    }
}
