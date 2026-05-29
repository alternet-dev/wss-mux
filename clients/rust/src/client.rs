//! Public client surface plus the drive task that owns the WebSocket.
//!
//! Design:
//! - `WssMuxClient` is a cheap handle. It holds a `mpsc::Sender<Command>`
//!   into a single drive task that owns the WebSocket. The drive task
//!   reads incoming frames and processes outgoing commands in a single
//!   `tokio::select!`.
//! - Subscriptions are tracked by the drive task. On reconnect they are
//!   replayed; on a fatal per-subscription error frame the corresponding
//!   `Subscription` receiver is closed.
//! - Token refresh: the user supplies a `get_token` callback. It is
//!   invoked on the initial connect and on close-code 4401; cached and
//!   reused on other reconnects.
//!
//! The drive task is the only owner of the WS halves; the public surface
//! never touches them directly. That keeps the locking story trivial —
//! all WS access is single-threaded inside the task.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

use crate::error::WssMuxError;
use crate::types::{
    ClientFrame, ErrorCode, PublishId, ServerFrame, SubscriptionId, CLOSE_BAD_FRAME,
    CLOSE_UNAUTHENTICATED, SUBPROTOCOL_JSON,
};
use crate::EventFrame;

use futures_util::stream::FuturesUnordered;
use serde_json::Value;

/// Reconnect/backoff configuration. Defaults match the TypeScript SDK.
#[derive(Debug, Clone)]
pub struct ReconnectOptions {
    /// Cap on reconnect attempts. `None` = no cap.
    pub max_attempts: Option<u32>,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
}

impl Default for ReconnectOptions {
    fn default() -> Self {
        Self {
            max_attempts: None,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Builder for [`WssMuxClient`].
pub struct ClientBuilder {
    url: Option<String>,
    get_token: Option<BoxedTokenSource>,
    reconnect: ReconnectOptions,
    publish_settle: Duration,
}

/// Default time `publish()` waits for a possible error frame before
/// resolving. Matches the TypeScript SDK's `publishSettleMs` default.
const DEFAULT_PUBLISH_SETTLE: Duration = Duration::from_millis(250);

/// Boxed async token source. The closure is invoked on initial connect
/// and on reconnect after close code 4401.
type BoxedTokenSource = Box<
    dyn FnMut() -> Pin<Box<dyn Future<Output = Result<String, WssMuxError>> + Send>>
        + Send
        + 'static,
>;

impl ClientBuilder {
    fn new() -> Self {
        Self {
            url: None,
            get_token: None,
            reconnect: ReconnectOptions::default(),
            publish_settle: DEFAULT_PUBLISH_SETTLE,
        }
    }

    /// Full WebSocket URL: `ws://host[:port]/path` or
    /// `wss://host[:port]/path`.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Async callback that returns the current auth token.
    pub fn get_token<F, Fut>(mut self, mut f: F) -> Self
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<String, WssMuxError>> + Send + 'static,
    {
        self.get_token = Some(Box::new(move || Box::pin(f())));
        self
    }

    /// Override reconnect behavior. Defaults are sensible for cloud
    /// deployments; tune for tighter retry budgets in tests.
    pub fn reconnect(mut self, opts: ReconnectOptions) -> Self {
        self.reconnect = opts;
        self
    }

    /// How long [`WssMuxClient::publish`] waits for a possible error
    /// frame before resolving. Ack-by-absence: within this window
    /// either an error arrives (reject) or the publish is accepted.
    /// Default 250 ms — comfortably above typical wide-area RTT.
    pub fn publish_settle(mut self, dur: Duration) -> Self {
        self.publish_settle = dur;
        self
    }

    /// Connect and authenticate, returning a client handle.
    ///
    /// Returns once the WebSocket is open and the auth frame has been
    /// written. Auth is acked by absence — a 4401 close shortly after
    /// indicates the token was rejected; that surfaces on the next
    /// `subscribe()` call or via the subscription's `recv()`.
    pub async fn build(self) -> Result<WssMuxClient, WssMuxError> {
        let url = self
            .url
            .ok_or_else(|| WssMuxError::Usage("url is required".into()))?;
        let get_token = self
            .get_token
            .ok_or_else(|| WssMuxError::Usage("get_token is required".into()))?;

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<Command>();
        let (ready_tx, ready_rx) = oneshot::channel::<Result<(), WssMuxError>>();

        let drive = Drive {
            url,
            get_token,
            reconnect: self.reconnect,
            publish_settle: self.publish_settle,
            cmd_rx,
            cached_token: None,
            subs: HashMap::new(),
            next_sub_counter: 1,
            pending_pubs: HashMap::new(),
            next_pub_counter: 1,
        };
        let join = tokio::spawn(drive.run(ready_tx));

        // Block on the initial connect attempt. After this returns the
        // drive task owns the WS and stays alive for subscribe/close.
        ready_rx
            .await
            .map_err(|_| WssMuxError::Transport("drive task aborted before ready".into()))??;

        Ok(WssMuxClient {
            inner: Arc::new(ClientInner {
                cmd_tx,
                drive_join: Mutex::new(Some(join)),
            }),
        })
    }
}

/// Handle to the wss-mux SDK. Cheap to clone — internally `Arc`-shared.
#[derive(Clone)]
pub struct WssMuxClient {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    cmd_tx: mpsc::UnboundedSender<Command>,
    drive_join: Mutex<Option<JoinHandle<()>>>,
}

impl WssMuxClient {
    /// Create a new builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Subscribe to events on `stream`, optionally narrowed by `key`.
    ///
    /// Returns a [`Subscription`] whose `recv()` yields incoming events
    /// (or per-sub errors). Dropping the subscription unsubscribes on
    /// the server side.
    pub async fn subscribe(
        &self,
        stream: impl Into<String>,
        key: Option<&str>,
    ) -> Result<Subscription, WssMuxError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.inner
            .cmd_tx
            .send(Command::Subscribe {
                stream: stream.into(),
                key: key.map(str::to_owned),
                drop_tx: self.inner.cmd_tx.clone(),
                resp: resp_tx,
            })
            .map_err(|_| WssMuxError::Closed)?;
        resp_rx.await.map_err(|_| WssMuxError::Closed)?
    }

    /// Emit an event on `stream`. Identical downstream semantics to
    /// HTTP `POST /events` — same per-subscription delivery, same peer
    /// fanout. The connection's principals must intersect the stream's
    /// `publish` audience on the server; otherwise the call rejects
    /// with [`WssMuxError::Protocol`] (`unauthorized_publish`).
    ///
    /// Ack-by-absence: the server does not send a success frame. The
    /// returned future resolves once the configured publish settle
    /// window elapses without an error frame matching the publish's
    /// id. If an error arrives in the window the future rejects with
    /// the matching `Protocol` error. If the connection drops while
    /// pending, it rejects with [`WssMuxError::ConnectionClosed`].
    pub async fn publish(
        &self,
        stream: impl Into<String>,
        key: Option<&str>,
        payload: Value,
    ) -> Result<(), WssMuxError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.inner
            .cmd_tx
            .send(Command::Publish {
                stream: stream.into(),
                key: key.map(str::to_owned),
                payload,
                resp: resp_tx,
            })
            .map_err(|_| WssMuxError::Closed)?;
        resp_rx.await.map_err(|_| WssMuxError::Closed)?
    }

    /// Graceful close. After this call no more operations are accepted.
    /// Active subscriptions' receivers will return `None`.
    pub async fn close(self) -> Result<(), WssMuxError> {
        // Best-effort: if drive task already gone, this is a no-op.
        let (ack_tx, ack_rx) = oneshot::channel();
        let _ = self.inner.cmd_tx.send(Command::Close { ack: ack_tx });
        // Wait for the drive task's ack; if it dropped without acking,
        // we still consider close complete.
        let _ = ack_rx.await;
        if let Some(handle) = self.inner.drive_join.lock().await.take() {
            // Bound the join so a stuck drive task can't wedge close().
            let _ = tokio::time::timeout(Duration::from_secs(1), handle).await;
        }
        Ok(())
    }
}

/// Owned event stream for a single subscription. On drop the SDK sends
/// an unsubscribe over the wire (best-effort — if the connection is
/// already gone, the drop is silent).
pub struct Subscription {
    id: SubscriptionId,
    rx: mpsc::UnboundedReceiver<Result<EventFrame, WssMuxError>>,
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl Subscription {
    /// Subscription id assigned by the SDK.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Receive the next event or per-subscription error. Returns `None`
    /// when the subscription is terminated (server-side fatal error,
    /// client close, or unsubscribe).
    pub async fn recv(&mut self) -> Option<Result<EventFrame, WssMuxError>> {
        self.rx.recv().await
    }

    /// Explicitly unsubscribe. Idempotent; equivalent to dropping the
    /// subscription but lets the caller await acknowledgement that the
    /// drive task has processed the command.
    pub async fn unsubscribe(mut self) -> Result<(), WssMuxError> {
        let id = std::mem::take(&mut self.id);
        if id.is_empty() {
            return Ok(());
        }
        // Send via the drop path so close-then-drop is a single behavior.
        self.send_unsub(&id);
        Ok(())
    }

    fn send_unsub(&self, id: &str) {
        let _ = self.cmd_tx.send(Command::Unsubscribe { id: id.into() });
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if !self.id.is_empty() {
            let id = std::mem::take(&mut self.id);
            self.send_unsub(&id);
        }
    }
}

// --- internals ---

enum Command {
    Subscribe {
        stream: String,
        key: Option<String>,
        /// A clone of the public `cmd_tx`. Stashed into the new
        /// `Subscription` so its `Drop` impl can post `Unsubscribe`
        /// back through the same channel. Cloning into the command
        /// (rather than holding a permanent clone in `Drive`) keeps
        /// the drop semantics clean: when every public handle is
        /// gone, the only senders left are the per-subscription
        /// clones, and once those drop the receiver closes and the
        /// drive task exits.
        drop_tx: mpsc::UnboundedSender<Command>,
        resp: oneshot::Sender<Result<Subscription, WssMuxError>>,
    },
    Unsubscribe {
        id: SubscriptionId,
    },
    Publish {
        stream: String,
        key: Option<String>,
        payload: Value,
        resp: oneshot::Sender<Result<(), WssMuxError>>,
    },
    Close {
        ack: oneshot::Sender<()>,
    },
}

struct ActiveSub {
    stream: String,
    key: Option<String>,
    tx: mpsc::UnboundedSender<Result<EventFrame, WssMuxError>>,
}

struct Drive {
    url: String,
    get_token: BoxedTokenSource,
    reconnect: ReconnectOptions,
    publish_settle: Duration,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
    cached_token: Option<String>,
    subs: HashMap<SubscriptionId, ActiveSub>,
    next_sub_counter: u64,
    /// Publishes that have been written to the wire but neither rejected
    /// by an error frame nor passed their settle window. Resolved when
    /// the settle timer fires; rejected when a matching error arrives or
    /// the connection drops.
    pending_pubs: HashMap<PublishId, oneshot::Sender<Result<(), WssMuxError>>>,
    next_pub_counter: u64,
}

/// Settle timer: yields a publish id when its settle window elapses.
type SettleTimer = Pin<Box<dyn Future<Output = PublishId> + Send>>;

impl Drive {
    async fn run(mut self, ready: oneshot::Sender<Result<(), WssMuxError>>) {
        // Initial connect.
        let mut conn = match self.connect(true).await {
            Ok(c) => {
                let _ = ready.send(Ok(()));
                c
            }
            Err(e) => {
                let _ = ready.send(Err(e));
                return;
            }
        };

        // Outer loop: each iteration drives one connection until it
        // closes, then reconnects (or exits if exhausted).
        let mut reconnect_attempt: u32 = 0;
        loop {
            let outcome = self.run_one_connection(&mut conn).await;
            match outcome {
                ConnectionOutcome::Closed => {
                    self.fail_all_pending_pubs(WssMuxError::Closed);
                    break;
                }
                ConnectionOutcome::ClosedByServer { code, reason } => {
                    let refresh = code == CLOSE_UNAUTHENTICATED;

                    // Pending publishes can't be acked across a fresh
                    // connection — the server has no record of them.
                    // Reject immediately rather than letting the settle
                    // timer eventually resolve them to false-positive Ok.
                    self.fail_all_pending_pubs(WssMuxError::ConnectionClosed {
                        code,
                        reason: reason.clone(),
                    });

                    if code == CLOSE_BAD_FRAME {
                        // Protocol bug on our side; don't loop reconnects.
                        self.fail_all_subs(WssMuxError::ConnectionClosed { code, reason });
                        return;
                    }

                    if let Some(cap) = self.reconnect.max_attempts {
                        if reconnect_attempt >= cap {
                            self.fail_all_subs(WssMuxError::ReconnectExhausted);
                            return;
                        }
                    }

                    let backoff = backoff_at(reconnect_attempt, &self.reconnect);
                    reconnect_attempt = reconnect_attempt.saturating_add(1);
                    tokio::time::sleep(backoff).await;

                    match self.connect(refresh).await {
                        Ok(c) => {
                            conn = c;
                            reconnect_attempt = 0;
                            // Replay subscriptions.
                            for (id, sub) in &self.subs {
                                let frame = ClientFrame::Subscribe {
                                    id: id.clone(),
                                    stream: sub.stream.clone(),
                                    key: sub.key.clone(),
                                };
                                let _ = send_frame(&mut conn, &frame).await;
                            }
                        }
                        Err(_) => {
                            // Fall through; next iteration will see no
                            // connection and try again or give up.
                            continue;
                        }
                    }
                }
            }
        }
    }

    /// Drive a single connection until it closes for any reason.
    async fn run_one_connection(&mut self, conn: &mut Connection) -> ConnectionOutcome {
        // Per-connection settle timers. A FuturesUnordered lets the
        // select loop wake on the next-firing timer without polling.
        // On reconnect we drop the whole set — any timers that hadn't
        // fired by then belong to publishes whose pending entry was
        // already rejected by the disconnect path.
        let mut settle_timers: FuturesUnordered<SettleTimer> = FuturesUnordered::new();
        // FuturesUnordered::next() returns None when empty AND keeps
        // returning None thereafter, so the select arm would spin.
        // The pending() future never resolves; we swap to it whenever
        // the timer set is empty.
        loop {
            tokio::select! {
                cmd = self.cmd_rx.recv() => match cmd {
                    Some(Command::Subscribe { stream, key, drop_tx, resp }) => {
                        let id = self.next_sub_id();
                        let (tx, rx) = mpsc::unbounded_channel();
                        self.subs.insert(id.clone(), ActiveSub {
                            stream: stream.clone(),
                            key: key.clone(),
                            tx,
                        });
                        let frame = ClientFrame::Subscribe { id: id.clone(), stream, key };
                        if let Err(e) = send_frame(conn, &frame).await {
                            self.subs.remove(&id);
                            let _ = resp.send(Err(e));
                            return ConnectionOutcome::ClosedByServer {
                                code: 1006,
                                reason: "write failed".into(),
                            };
                        }
                        let sub = Subscription {
                            id,
                            rx,
                            cmd_tx: drop_tx,
                        };
                        let _ = resp.send(Ok(sub));
                    }
                    Some(Command::Unsubscribe { id }) => {
                        if self.subs.remove(&id).is_some() {
                            let frame = ClientFrame::Unsubscribe { id };
                            let _ = send_frame(conn, &frame).await;
                        }
                    }
                    Some(Command::Publish { stream, key, payload, resp }) => {
                        let id = self.next_pub_id();
                        let frame = ClientFrame::Publish {
                            id: id.clone(),
                            stream,
                            key,
                            payload,
                        };
                        if let Err(e) = send_frame(conn, &frame).await {
                            let _ = resp.send(Err(e));
                            return ConnectionOutcome::ClosedByServer {
                                code: 1006,
                                reason: "write failed".into(),
                            };
                        }
                        self.pending_pubs.insert(id.clone(), resp);
                        let dur = self.publish_settle;
                        settle_timers.push(Box::pin(async move {
                            tokio::time::sleep(dur).await;
                            id
                        }));
                    }
                    Some(Command::Close { ack }) => {
                        // Best-effort clean close frame; ignore errors.
                        let _ = conn.ws.send(Message::Close(Some(CloseFrame {
                            code: CloseCode::Normal,
                            reason: "client close".into(),
                        }))).await;
                        let _ = ack.send(());
                        return ConnectionOutcome::Closed;
                    }
                    None => {
                        // All client handles dropped — close.
                        return ConnectionOutcome::Closed;
                    }
                },
                // Settle next-firing publish, if any. The fallback to
                // pending() keeps this arm dormant when no timers exist
                // (FuturesUnordered::next would otherwise yield None
                // immediately and spin the loop).
                settled = next_settle_or_pending(&mut settle_timers) => {
                    if let Some(resp) = self.pending_pubs.remove(&settled) {
                        let _ = resp.send(Ok(()));
                    }
                }
                msg = conn.ws.next() => match msg {
                    Some(Ok(Message::Text(t))) => {
                        if let Some(outcome) = self.dispatch_text(&t) {
                            return outcome;
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        // Ignore — the JSON subprotocol is text.
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // Respond explicitly. tungstenite surfaces Ping
                        // to the consumer and expects us to echo the
                        // payload back as Pong — the server's liveness
                        // tracker uses this to keep us in the registry.
                        if conn.ws.send(Message::Pong(data)).await.is_err() {
                            return ConnectionOutcome::ClosedByServer {
                                code: 1006,
                                reason: "pong write failed".into(),
                            };
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // The SDK doesn't initiate pings — Pong is unexpected
                        // but harmless. Ignore.
                    }
                    Some(Ok(Message::Close(frame))) => {
                        let (code, reason) = frame
                            .map(|f| (u16::from(f.code), f.reason.into_owned()))
                            .unwrap_or((1006, String::new()));
                        return ConnectionOutcome::ClosedByServer { code, reason };
                    }
                    Some(Ok(Message::Frame(_))) => { /* raw frame — unreachable in practice */ }
                    Some(Err(_)) => {
                        return ConnectionOutcome::ClosedByServer {
                            code: 1006,
                            reason: "ws error".into(),
                        };
                    }
                    None => {
                        return ConnectionOutcome::ClosedByServer {
                            code: 1006,
                            reason: "stream end".into(),
                        };
                    }
                },
            }
        }
    }

    /// Returns `Some(ClosedByServer)` if dispatch decided the connection
    /// should be torn down (e.g. server-fatal error). `None` otherwise.
    fn dispatch_text(&mut self, raw: &str) -> Option<ConnectionOutcome> {
        let frame: ServerFrame = match serde_json::from_str(raw) {
            Ok(f) => f,
            Err(_) => {
                // The server is well-behaved; ignore malformed frames
                // defensively rather than tearing down the connection.
                return None;
            }
        };
        match frame {
            ServerFrame::Event {
                id,
                stream,
                key,
                payload,
            } => {
                if let Some(sub) = self.subs.get(&id) {
                    let _ = sub.tx.send(Ok(EventFrame {
                        id,
                        stream,
                        key,
                        payload,
                    }));
                }
                None
            }
            ServerFrame::Error { code, message, id } => {
                if let Some(id_str) = id {
                    // Pending publish? Match by id and reject the
                    // awaited promise. Server publish errors all carry
                    // the originating publish's id back, so any
                    // matching id is treated as the publish ack.
                    if let Some(resp) = self.pending_pubs.remove(&id_str) {
                        let _ = resp.send(Err(WssMuxError::Protocol {
                            code,
                            message,
                            id: Some(id_str),
                        }));
                        return None;
                    }
                    if matches!(
                        code,
                        ErrorCode::UnknownStream
                            | ErrorCode::UnauthorizedSubscribe
                            | ErrorCode::DuplicateSubscriptionId
                            | ErrorCode::Overflow
                    ) {
                        if let Some(sub) = self.subs.remove(&id_str) {
                            let _ = sub.tx.send(Err(WssMuxError::Protocol {
                                code,
                                message,
                                id: Some(id_str),
                            }));
                            // Drop tx → receiver gets None on next recv
                        }
                    }
                }
                None
            }
        }
    }

    async fn connect(&mut self, refresh_token: bool) -> Result<Connection, WssMuxError> {
        if refresh_token || self.cached_token.is_none() {
            let token = (self.get_token)().await?;
            self.cached_token = Some(token);
        }
        let token = self
            .cached_token
            .clone()
            .ok_or_else(|| WssMuxError::Usage("token unavailable".into()))?;

        // Build the upgrade request and negotiate the JSON subprotocol.
        let mut req = self
            .url
            .as_str()
            .into_client_request()
            .map_err(|e| WssMuxError::Transport(format!("invalid URL: {e}")))?;
        req.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            SUBPROTOCOL_JSON
                .parse()
                .expect("static header value parses"),
        );

        let (ws, _resp) = tokio_tungstenite::connect_async(req)
            .await
            .map_err(|e| WssMuxError::Transport(e.to_string()))?;

        let mut conn = Connection { ws };

        // Send auth frame. Ack-by-absence; a 4401 close arrives later if
        // the token was rejected.
        send_frame(&mut conn, &ClientFrame::Auth { token }).await?;

        Ok(conn)
    }

    fn next_sub_id(&mut self) -> SubscriptionId {
        let id = format!("sub-{}", self.next_sub_counter);
        self.next_sub_counter += 1;
        id
    }

    fn next_pub_id(&mut self) -> PublishId {
        let id = format!("pub-{}", self.next_pub_counter);
        self.next_pub_counter += 1;
        id
    }

    fn fail_all_pending_pubs(&mut self, err: WssMuxError) {
        for (_, resp) in self.pending_pubs.drain() {
            let _ = resp.send(Err(err.clone()));
        }
    }

    fn fail_all_subs(&mut self, err: WssMuxError) {
        for (_, sub) in self.subs.drain() {
            let _ = sub.tx.send(Err(err.clone()));
        }
    }
}

// ----------------------------------------------------------------------

struct Connection {
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
}

enum ConnectionOutcome {
    /// Closed cleanly by the client (explicit `close()`).
    Closed,
    /// Closed by the server (or transport). Caller decides whether to
    /// reconnect based on the close code.
    ClosedByServer { code: u16, reason: String },
}

async fn send_frame(conn: &mut Connection, frame: &ClientFrame) -> Result<(), WssMuxError> {
    let json = serde_json::to_string(frame).map_err(|e| WssMuxError::Transport(e.to_string()))?;
    conn.ws
        .send(Message::Text(json))
        .await
        .map_err(map_ws_error)
}

fn map_ws_error(e: WsError) -> WssMuxError {
    match e {
        WsError::ConnectionClosed | WsError::AlreadyClosed => WssMuxError::Closed,
        other => WssMuxError::Transport(other.to_string()),
    }
}

/// Returns the next-settling publish id, or sleeps forever if the
/// timer set is empty (so the select arm stays parked instead of
/// busy-yielding `None` from an exhausted `FuturesUnordered`).
async fn next_settle_or_pending(set: &mut FuturesUnordered<SettleTimer>) -> PublishId {
    if set.is_empty() {
        std::future::pending::<PublishId>().await
    } else {
        match set.next().await {
            Some(id) => id,
            None => std::future::pending::<PublishId>().await,
        }
    }
}

fn backoff_at(attempt: u32, opts: &ReconnectOptions) -> Duration {
    let initial = opts.initial_backoff.as_secs_f64();
    let max = opts.max_backoff.as_secs_f64();
    let scaled = initial * opts.backoff_multiplier.powi(attempt as i32);
    Duration::from_secs_f64(scaled.min(max))
}

// Clone helper: WssMuxError's inner String for Protocol/Closed reasons
// is just String, so a manual Clone impl over the variants is mechanical.
impl Clone for WssMuxError {
    fn clone(&self) -> Self {
        match self {
            Self::Protocol { code, message, id } => Self::Protocol {
                code: *code,
                message: message.clone(),
                id: id.clone(),
            },
            Self::ConnectionClosed { code, reason } => Self::ConnectionClosed {
                code: *code,
                reason: reason.clone(),
            },
            Self::ReconnectExhausted => Self::ReconnectExhausted,
            Self::Closed => Self::Closed,
            Self::Usage(s) => Self::Usage(s.clone()),
            Self::Transport(s) => Self::Transport(s.clone()),
        }
    }
}
