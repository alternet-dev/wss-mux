//! Target selection: spawn wss-mux in-process, or point the harness at
//! an already-running instance.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use wss_mux::config::{Config, HandshakeKeyConfig};
use wss_mux::manifest::Manifest;
use wss_mux::server::{build_app, AppState};

/// Manifest the in-process server is brought up with: one `loadgen`
/// stream with a `["*"]` audience, so any minted token may subscribe.
pub const LOADGEN_MANIFEST: &str = r#"version: 1
streams:
  - stream: loadgen
    audience: ["*"]
"#;

/// Where the harness is pointed.
pub enum Target {
    /// wss-mux spawned in this process. `_server` holds the serve task
    /// for the lifetime of the run; the process exiting tears it down.
    InProcess {
        addr: SocketAddr,
        _server: JoinHandle<()>,
    },
    /// An already-running instance reached over the network.
    External { base_url: String },
}

impl Target {
    /// `http://…` base for the REST surface (`/v1/events`, `/metrics`).
    pub fn base_url(&self) -> String {
        match self {
            Target::InProcess { addr, .. } => format!("http://{addr}"),
            Target::External { base_url } => base_url.trim_end_matches('/').to_string(),
        }
    }

    /// `ws://…` (or `wss://…`) base for the WebSocket endpoint.
    pub fn ws_url(&self) -> String {
        match self {
            Target::InProcess { addr, .. } => format!("ws://{addr}"),
            Target::External { base_url } => {
                let trimmed = base_url.trim_end_matches('/');
                if let Some(rest) = trimmed.strip_prefix("https://") {
                    format!("wss://{rest}")
                } else if let Some(rest) = trimmed.strip_prefix("http://") {
                    format!("ws://{rest}")
                } else {
                    format!("ws://{trimmed}")
                }
            }
        }
    }

    /// Stamped onto every result so a number is never read out of context.
    pub fn mode(&self) -> &'static str {
        match self {
            Target::InProcess { .. } => "in-process",
            Target::External { .. } => "external",
        }
    }
}

/// Resolve the harness target. `Some(url)` ⇒ external; `None` ⇒ spawn
/// wss-mux in-process.
pub async fn start(target: Option<String>, signing_key: &str, push_token: &str) -> Result<Target> {
    match target {
        Some(base_url) => Ok(Target::External { base_url }),
        None => start_in_process(signing_key, push_token).await,
    }
}

/// Spawn wss-mux in-process on `127.0.0.1:0` with rate limiting off (the
/// harness measures the dispatch path, not the inbound limiter) and peer
/// relay disabled (single instance — there is nothing to relay to).
async fn start_in_process(signing_key: &str, push_token: &str) -> Result<Target> {
    let mut config = Config::new(
        push_token.to_string(),
        HandshakeKeyConfig {
            hs256_secret: Some(signing_key.to_string()),
            ed25519_public_pem: None,
        },
        PathBuf::from("loadgen.yaml"),
    );
    config.inbound_rate_per_sec = 0;
    config.inbound_burst = 0;
    config.peers.relay_enabled = false;

    let state = AppState::try_new(config).context("build in-process AppState")?;
    let manifest = Manifest::from_str(LOADGEN_MANIFEST, Path::new("loadgen.yaml"))
        .context("parse the in-process loadgen manifest")?;
    state.set_manifest(manifest);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind an ephemeral port for the in-process server")?;
    let addr = listener
        .local_addr()
        .context("read in-process server addr")?;

    let server = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, build_app(state)).await {
            tracing::error!(error = %e, "in-process wss-mux server exited");
        }
    });

    Ok(Target::InProcess {
        addr,
        _server: server,
    })
}
