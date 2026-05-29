use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use wss_mux::config::Config;
use wss_mux::manifest::Manifest;
use wss_mux::server::metrics::{ReloadResult, ReloadResultLabel};
use wss_mux::server::{build_app, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let manifest = Manifest::load(&config.manifest_path)?;
    tracing::info!(streams = manifest.streams.len(), "manifest loaded");

    let state = AppState::try_new(config)?;
    state.set_manifest(manifest);

    spawn_sighup_reloader(state.clone());
    spawn_peer_refresher(state.clone());
    spawn_oidc_jwks_refresher(state.clone());
    spawn_ws_publish_idle_sweeper(state.clone());
    wss_mux::server::relay::spawn_relay_flusher(state.clone());

    let listener = TcpListener::bind(state.config().listen_addr).await?;
    tracing::info!(addr = %listener.local_addr()?, "wss-mux listening");

    axum::serve(listener, build_app(state)).await?;
    Ok(())
}

/// Keep the peer set fresh. Membership is polled (not per-relay
/// delivery): re-resolve discovery every `WSS_MUX_PEER_DNS_REFRESH_SECS`
/// so scale-up/down propagates within a tight window. A static
/// `WSS_MUX_PEERS` fleet is fixed — set once, never polled. Relay
/// disabled (`WSS_MUX_PEER_RELAY=off`) ⇒ task not spawned, no DNS query.
fn spawn_peer_refresher(state: AppState) {
    let peer_cfg = state.config().peers.clone();
    if !peer_cfg.relay_enabled {
        tracing::info!("peer relay disabled (WSS_MUX_PEER_RELAY=off)");
        return;
    }
    let self_port = state.config().listen_addr.port();
    tokio::spawn(async move {
        loop {
            let peers = wss_mux::peers::discover(&peer_cfg, self_port).await;
            let count = peers.len();
            state.set_peers(peers);
            tracing::debug!(peers = count, "peer set refreshed");
            if !peer_cfg.static_peers.is_empty() {
                return; // fixed fleet — no polling
            }
            tokio::time::sleep(peer_cfg.dns_refresh).await;
        }
    });
}

/// Periodically drop idle per-source publish buckets so the keyed
/// limiter map can't grow unboundedly with one-off sources. Inert
/// (task not spawned) when the per-source WS publish limiter is
/// disabled. Sweep cadence is one tenth of the configured idle TTL —
/// frequent enough to keep evictions even under a steady churn, rare
/// enough that the sweep cost is amortized vs every publish.
fn spawn_ws_publish_idle_sweeper(state: AppState) {
    let Some(limiter) = state.ws_publish_limiter().cloned() else {
        return;
    };
    let ttl = state.config().ws_publish_idle_ttl;
    let cadence = (ttl / 10).max(std::time::Duration::from_secs(1));
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(cadence);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tick.tick().await;
            let evicted = limiter.sweep_idle(std::time::Instant::now());
            if evicted > 0 {
                tracing::debug!(
                    evicted,
                    active = limiter.active_sources(),
                    "ws-publish per-source sweep"
                );
            }
        }
    });
}

/// Keep the OIDC JWKS fresh. Not spawned unless `WSS_MUX_OIDC_ISSUER`
/// is set (OIDC disabled ⇒ inert, no outbound calls). Startup fetch,
/// then refresh every `WSS_MUX_OIDC_JWKS_REFRESH`. A failed fetch is
/// logged + metered and keeps the last-good cache — a transient IdP
/// blip must never 401 every client (same posture as the manifest
/// reloader and peer refresher; `ready` still gates a never-fetched
/// JWKS out of rotation).
fn spawn_oidc_jwks_refresher(state: AppState) {
    let Some(oidc) = state.config().oidc.clone() else {
        return;
    };
    tokio::spawn(async move {
        let http = reqwest::Client::new();
        loop {
            match wss_mux::oidc::fetch_jwks(&http, &oidc).await {
                Ok(set) => {
                    let keys = set.keys.len();
                    state.set_jwks(set);
                    state
                        .metrics()
                        .oidc_jwks_refresh
                        .get_or_create(&ReloadResultLabel {
                            result: ReloadResult::Ok,
                        })
                        .inc();
                    tracing::info!(keys, "oidc jwks refreshed");
                }
                Err(e) => {
                    state
                        .metrics()
                        .oidc_jwks_refresh
                        .get_or_create(&ReloadResultLabel {
                            result: ReloadResult::Error,
                        })
                        .inc();
                    tracing::error!(error = %e, "oidc jwks refresh failed; keeping last-good");
                }
            }
            tokio::time::sleep(oidc.jwks_refresh).await;
        }
    });
}

/// Reload the manifest from disk on every SIGHUP. A failed reload logs and
/// meters the error but keeps the previously loaded manifest serving — a
/// bad edit must never take the process down.
fn spawn_sighup_reloader(state: AppState) {
    tokio::spawn(async move {
        let mut sighup = match signal(SignalKind::hangup()) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "failed to install SIGHUP handler");
                return;
            }
        };
        let path = state.config().manifest_path.clone();
        while sighup.recv().await.is_some() {
            match state.reload_manifest(&path) {
                Ok(streams) => {
                    state
                        .metrics()
                        .manifest_reloads
                        .get_or_create(&ReloadResultLabel {
                            result: ReloadResult::Ok,
                        })
                        .inc();
                    tracing::info!(streams, "manifest reloaded via SIGHUP");
                }
                Err(e) => {
                    state
                        .metrics()
                        .manifest_reloads
                        .get_or_create(&ReloadResultLabel {
                            result: ReloadResult::Error,
                        })
                        .inc();
                    tracing::error!(error = %e, "manifest reload failed; keeping previous manifest");
                }
            }
        }
    });
}
