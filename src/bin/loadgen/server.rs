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

/// Manifest every in-process instance is brought up with. All streams
/// carry a `["*"]` audience, so any minted token may subscribe.
/// `loadgen_slow` has a tiny send queue (slow-consumer scenario);
/// `loadgen_capped` carries a payload cap (payload-cap scenario).
pub const LOADGEN_MANIFEST: &str = r#"version: 1
streams:
  - stream: loadgen
    audience: ["*"]
  - stream: loadgen_slow
    audience: ["*"]
    queue_depth: 8
  - stream: loadgen_capped
    audience: ["*"]
    max_payload_bytes: 256
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

    /// Number of instances in the fleet (always ≥ 1).
    pub fn fleet_size(&self) -> usize {
        self.instances.len()
    }

    /// Index of the instance the `concentrated` topology pins every
    /// subscriber to, and the one a single-instance run uses: instance
    /// 1 in a fleet, instance 0 otherwise.
    pub fn subscriber_index(&self) -> usize {
        self.instances.len().min(2) - 1
    }

    /// REST base of the ingress instance — where producers push.
    pub fn producer_base(&self) -> &str {
        self.instances[0].base_url.as_str()
    }

    /// REST base of instance `idx`.
    pub fn instance_base(&self, idx: usize) -> &str {
        self.instances[idx].base_url.as_str()
    }

    /// WebSocket base of instance `idx`.
    pub fn instance_ws(&self, idx: usize) -> &str {
        self.instances[idx].ws_url.as_str()
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

/// Per-scenario knobs for a single profiled in-process instance — the
/// traffic-oddity scenarios each request the variant they need.
pub struct ServerProfile {
    /// Inbound frame rate limit `(per_sec, burst)`; `(0, 0)` ⇒ off.
    pub rate_limit: (u32, u32),
    /// Relay coalescing window, ms; `0` ⇒ off (direct relay).
    pub coalesce_ms: u64,
    /// Bounded relay-coalescing queue depth.
    pub relay_queue_depth: usize,
    /// Configure one guaranteed-dead peer, so every relay fails fast.
    pub dead_peer: bool,
}

impl Default for ServerProfile {
    fn default() -> Self {
        ServerProfile {
            rate_limit: (0, 0),
            coalesce_ms: 0,
            relay_queue_depth: 1024,
            dead_peer: false,
        }
    }
}

/// Spawn a single in-process wss-mux instance configured for a stress
/// scenario, and return it as a one-instance `Target`.
pub async fn start_profiled(
    profile: ServerProfile,
    signing_key: &str,
    push_token: &str,
) -> Result<Target> {
    let mut config = Config::new(
        push_token.to_string(),
        HandshakeKeyConfig {
            hs256_secret: Some(signing_key.to_string()),
            ed25519_public_pem: None,
        },
        PathBuf::from("loadgen.yaml"),
    );
    config.inbound_rate_per_sec = profile.rate_limit.0;
    config.inbound_burst = profile.rate_limit.1;
    config.relay_coalesce_ms = profile.coalesce_ms;
    config.relay_queue_depth = profile.relay_queue_depth.max(1);

    let state = AppState::try_new(config).context("build profiled AppState")?;
    let manifest = Manifest::from_str(LOADGEN_MANIFEST, Path::new("loadgen.yaml"))
        .context("parse the loadgen manifest")?;
    state.set_manifest(manifest);

    if profile.dead_peer {
        // Bind then drop a port: connections to it are refused fast.
        let dead = TcpListener::bind("127.0.0.1:0")
            .await
            .context("bind a port for the dead peer")?;
        let dead_addr = dead.local_addr().context("read dead peer addr")?;
        drop(dead);
        state.set_peers(vec![PeerUrl {
            scheme: Scheme::Http,
            host: dead_addr.ip().to_string(),
            port: dead_addr.port(),
        }]);
    }

    if profile.coalesce_ms > 0 {
        wss_mux::server::relay::spawn_relay_flusher(state.clone());
    }

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind the profiled instance")?;
    let addr = listener.local_addr().context("read profiled addr")?;
    let server = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, build_app(state)).await {
            tracing::error!(error = %e, "profiled wss-mux instance exited");
        }
    });

    Ok(Target {
        instances: vec![Instance::in_process(addr)],
        mode: "in-process",
        _servers: vec![server],
    })
}
