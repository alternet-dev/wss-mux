//! The relay HTTP client (per-relay timeout + optional peer TLS).

use anyhow::Context;

use crate::config::PeerConfig;

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
