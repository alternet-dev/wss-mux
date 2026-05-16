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

    let state = AppState::new(config);
    state.set_manifest(manifest);

    spawn_sighup_reloader(state.clone());

    let listener = TcpListener::bind(state.config().listen_addr).await?;
    tracing::info!(addr = %listener.local_addr()?, "wss-mux listening");

    axum::serve(listener, build_app(state)).await?;
    Ok(())
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
