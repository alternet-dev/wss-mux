//! Peer base-URL value type and parsing.

use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
