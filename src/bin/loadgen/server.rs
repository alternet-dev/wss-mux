//! Target selection: spawn a wss-mux fleet in-process, or point the
//! harness at an already-running instance.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use wss_mux::config::{Config, HandshakeKeyConfig};
use wss_mux::manifest::Manifest;
use wss_mux::peers::{PeerUrl, Scheme};
use wss_mux::server::{build_app, AppState};

/// Manifest every in-process instance is brought up with: one `loadgen`
/// stream with a `["*"]` audience, so any minted token may subscribe.
pub const LOADGEN_MANIFEST: &str = r#"version: 1
streams:
  - stream: loadgen
    audience: ["*"]
"#;

/// One reachable wss-mux instance.
struct Instance {
    /// `http://host:port` — REST surface (`/v1/events`, `/metrics`).
    base_url: String,
    /// `ws://host:port` (or `wss://…`) — the WebSocket endpoint.
    ws_url: String,
}

impl Instance {
    /// An external instance from a user-supplied base URL.
    fn external(base_url: &str) -> Instance {
        let base = base_url.trim_end_matches('/').to_string();
        let ws_url = if let Some(rest) = base.strip_prefix("https://") {
            format!("wss://{rest}")
        } else if let Some(rest) = base.strip_prefix("http://") {
            format!("ws://{rest}")
        } else {
            format!("ws://{base}")
        };
        Instance {
            base_url: base,
            ws_url,
        }
    }

    /// An in-process instance from its bound loopback address.
    fn in_process(addr: SocketAddr) -> Instance {
        Instance {
            base_url: format!("http://{addr}"),
            ws_url: format!("ws://{addr}"),
        }
    }
}

/// Where the harness is pointed: a fleet of one or more instances. With
/// a fleet (`> 1` instance), instance 0 is the producer ingress and
/// instance 1 is where subscribers connect — so the measured path
/// crosses a peer-relay hop.
pub struct Target {
    instances: Vec<Instance>,
    mode: &'static str,
    // In-process serve tasks, held for the lifetime of the run; the
    // process exiting tears them down. Empty in external mode.
    _servers: Vec<JoinHandle<()>>,
}

impl Target {
    /// `in-process` or `external`.
    pub fn mode(&self) -> &'static str {
        self.mode
    }

    /// Peer count — fleet size minus the ingress instance. `0` ⇒ a
    /// single instance; this is the `--peers` value the run used.
    pub fn peers(&self) -> usize {
        self.instances.len() - 1
    }

    /// REST base of the ingress instance — where producers push.
    pub fn producer_base(&self) -> &str {
        self.instances[0].base_url.as_str()
    }

    /// The instance subscribers connect to: instance 1 in a fleet (so
    /// the path crosses a relay hop), else the sole instance.
    fn subscriber(&self) -> &Instance {
        &self.instances[self.instances.len().min(2) - 1]
    }

    /// WebSocket base of the subscriber instance.
    pub fn subscriber_ws(&self) -> &str {
        self.subscriber().ws_url.as_str()
    }

    /// REST base of the subscriber instance — for the readiness scrape.
    pub fn subscriber_base(&self) -> &str {
        self.subscriber().base_url.as_str()
    }

    /// REST bases of every instance — for fleet-wide metric scraping.
    pub fn instance_bases(&self) -> Vec<&str> {
        self.instances.iter().map(|i| i.base_url.as_str()).collect()
    }
}

/// Resolve the harness target. `Some(url)` ⇒ external (a single
/// instance; `peers` is in-process-only and ignored with a warning).
/// `None` ⇒ spawn an in-process fleet of `peers + 1` instances.
pub async fn start(
    target: Option<String>,
    peers: usize,
    signing_key: &str,
    push_token: &str,
) -> Result<Target> {
    match target {
        Some(base_url) => {
            if peers > 0 {
                tracing::warn!("--peers is in-process-only; driving the single external target");
            }
            Ok(Target {
                instances: vec![Instance::external(&base_url)],
                mode: "external",
                _servers: Vec::new(),
            })
        }
        None => start_fleet(peers + 1, signing_key, push_token).await,
    }
}

/// Spawn `count` wss-mux instances in-process, each on its own ephemeral
/// `127.0.0.1` port, every instance configured with the others as peers
/// so a producer push relays across the fleet. Rate limiting is off —
/// the harness measures dispatch and relay, not the inbound limiter.
async fn start_fleet(count: usize, signing_key: &str, push_token: &str) -> Result<Target> {
    let count = count.max(1);

    // Bind every listener first so each instance can be told the others'
    // addresses before it starts serving.
    let mut listeners = Vec::with_capacity(count);
    let mut addrs = Vec::with_capacity(count);
    for _ in 0..count {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("bind an ephemeral port for an in-process instance")?;
        addrs.push(listener.local_addr().context("read in-process addr")?);
        listeners.push(listener);
    }

    let manifest = Manifest::from_str(LOADGEN_MANIFEST, Path::new("loadgen.yaml"))
        .context("parse the in-process loadgen manifest")?;

    let mut instances = Vec::with_capacity(count);
    let mut servers = Vec::with_capacity(count);
    for (i, listener) in listeners.into_iter().enumerate() {
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

        let state = AppState::try_new(config).context("build in-process AppState")?;
        state.set_manifest(manifest.clone());
        // Every other instance is a peer. With a single instance the
        // peer set is empty ⇒ spawn_relay is inert (PR-4 behaviour).
        let peer_urls: Vec<PeerUrl> = addrs
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, a)| PeerUrl {
                scheme: Scheme::Http,
                host: a.ip().to_string(),
                port: a.port(),
            })
            .collect();
        state.set_peers(peer_urls);

        instances.push(Instance::in_process(addrs[i]));
        servers.push(tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, build_app(state)).await {
                tracing::error!(error = %e, "in-process wss-mux instance exited");
            }
        }));
    }

    Ok(Target {
        instances,
        mode: "in-process",
        _servers: servers,
    })
}
