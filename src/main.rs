use tokio::net::TcpListener;
use wss_mux::config::Config;
use wss_mux::manifest::Manifest;
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

    let listen_addr = config.listen_addr;
    let state = AppState::new(config);
    state
        .set_manifest(manifest)
        .expect("manifest set exactly once at startup");

    let listener = TcpListener::bind(listen_addr).await?;
    tracing::info!(addr = %listener.local_addr()?, "wss-mux listening");

    axum::serve(listener, build_app(state)).await?;
    Ok(())
}
