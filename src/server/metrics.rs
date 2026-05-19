use std::fmt::Write;
use std::sync::Mutex;

use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue, LabelValueEncoder};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct StreamLabel {
    pub stream: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct DropReasonLabel {
    pub reason: DropReason,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum DropReason {
    Overflow,
    NoSubscribers,
    ChannelClosed,
}

impl EncodeLabelValue for DropReason {
    fn encode(&self, encoder: &mut LabelValueEncoder<'_>) -> Result<(), std::fmt::Error> {
        let s = match self {
            Self::Overflow => "overflow",
            Self::NoSubscribers => "no_subscribers",
            Self::ChannelClosed => "channel_closed",
        };
        encoder.write_str(s)
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct ReloadResultLabel {
    pub result: ReloadResult,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum ReloadResult {
    Ok,
    Error,
}

impl EncodeLabelValue for ReloadResult {
    fn encode(&self, encoder: &mut LabelValueEncoder<'_>) -> Result<(), std::fmt::Error> {
        encoder.write_str(match self {
            Self::Ok => "ok",
            Self::Error => "error",
        })
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct RevokeReasonLabel {
    pub reason: RevokeReason,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum RevokeReason {
    UnknownStream,
    Unauthorized,
}

impl EncodeLabelValue for RevokeReason {
    fn encode(&self, encoder: &mut LabelValueEncoder<'_>) -> Result<(), std::fmt::Error> {
        encoder.write_str(match self {
            Self::UnknownStream => "unknown_stream",
            Self::Unauthorized => "unauthorized",
        })
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct RejectReasonLabel {
    pub reason: RejectReason,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum RejectReason {
    /// Event payload exceeded the stream's `max_payload_bytes` cap.
    PayloadTooLarge,
}

impl EncodeLabelValue for RejectReason {
    fn encode(&self, encoder: &mut LabelValueEncoder<'_>) -> Result<(), std::fmt::Error> {
        encoder.write_str(match self {
            Self::PayloadTooLarge => "payload_too_large",
        })
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct RelayFailureLabel {
    pub reason: RelayFailure,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum RelayFailure {
    /// Per-relay request exceeded `WSS_MUX_PEER_RELAY_TIMEOUT_MS`.
    Timeout,
    /// Could not establish a connection (refused / unreachable / DNS).
    Connect,
    /// Peer answered with a non-success HTTP status.
    Status,
    /// Anything else (encode error, body error, ...).
    Other,
}

impl EncodeLabelValue for RelayFailure {
    fn encode(&self, encoder: &mut LabelValueEncoder<'_>) -> Result<(), std::fmt::Error> {
        encoder.write_str(match self {
            Self::Timeout => "timeout",
            Self::Connect => "connect",
            Self::Status => "status",
            Self::Other => "other",
        })
    }
}

pub struct Metrics {
    registry: Mutex<Registry>,
    pub connections_total: Counter,
    pub connections_active: Gauge,
    pub subscriptions_active: Gauge,
    pub events_dispatched: Family<StreamLabel, Counter>,
    pub events_dropped: Family<DropReasonLabel, Counter>,
    pub send_queue_depth: Histogram,
    pub manifest_reloads: Family<ReloadResultLabel, Counter>,
    pub subscriptions_revoked: Family<RevokeReasonLabel, Counter>,
    pub frames_rate_limited: Counter,
    pub relay_sent: Counter,
    /// Relayed (peer-origin) events that matched no local subscription
    /// — the cross-instance-waste / stream-sparsity signal that gates
    /// the deferred Selective Relay.
    pub relay_events_unwanted: Counter,
    /// Relay coalescing flush cycles (one per window/size flush).
    pub relay_flushes: Counter,
    /// Events handed to the peer-relay path (enqueued or direct).
    pub relay_events_relayed: Counter,
    /// Current depth of the relay coalescing queue.
    pub relay_queue_depth: Gauge,
    /// Relay batches dropped because the coalescing queue was full.
    pub relay_queue_dropped: Counter,
    /// Relay flush-task supervised restarts (should stay 0).
    pub relay_flush_restarts: Counter,
    pub relay_failed: Family<RelayFailureLabel, Counter>,
    pub peers_known: Gauge,
    pub events_rejected: Family<RejectReasonLabel, Counter>,
    /// Keys currently in the cached OIDC JWKS (`0` until first fetch).
    pub oidc_jwks_keys: Gauge,
    /// OIDC JWKS refresh attempts, labeled by result (reuses the
    /// manifest-reload Ok/Error label).
    pub oidc_jwks_refresh: Family<ReloadResultLabel, Counter>,
}

impl Default for Metrics {
    fn default() -> Self {
        let mut registry = Registry::default();

        let connections_total = Counter::default();
        let connections_active = Gauge::default();
        let subscriptions_active = Gauge::default();
        let events_dispatched = Family::<StreamLabel, Counter>::default();
        let events_dropped = Family::<DropReasonLabel, Counter>::default();
        let send_queue_depth = Histogram::new([1.0, 4.0, 16.0, 64.0, 256.0, 1024.0].into_iter());
        let manifest_reloads = Family::<ReloadResultLabel, Counter>::default();
        let subscriptions_revoked = Family::<RevokeReasonLabel, Counter>::default();
        let frames_rate_limited = Counter::default();
        let relay_sent = Counter::default();
        let relay_events_unwanted = Counter::default();
        let relay_flushes = Counter::default();
        let relay_events_relayed = Counter::default();
        let relay_queue_depth = Gauge::default();
        let relay_queue_dropped = Counter::default();
        let relay_flush_restarts = Counter::default();
        let relay_failed = Family::<RelayFailureLabel, Counter>::default();
        let peers_known = Gauge::default();
        let events_rejected = Family::<RejectReasonLabel, Counter>::default();
        let oidc_jwks_keys = Gauge::default();
        let oidc_jwks_refresh = Family::<ReloadResultLabel, Counter>::default();

        registry.register(
            "wss_mux_connections",
            "Total connections accepted since process start",
            connections_total.clone(),
        );
        registry.register(
            "wss_mux_connections_active",
            "Currently open WebSocket connections",
            connections_active.clone(),
        );
        registry.register(
            "wss_mux_subscriptions_active",
            "Currently active subscriptions across all connections",
            subscriptions_active.clone(),
        );
        registry.register(
            "wss_mux_events_dispatched",
            "Events successfully enqueued for delivery, labeled by stream",
            events_dispatched.clone(),
        );
        registry.register(
            "wss_mux_events_dropped",
            "Events dropped without delivery, labeled by reason",
            events_dropped.clone(),
        );
        registry.register(
            "wss_mux_send_queue_depth",
            "Per-connection send queue occupancy observed at dispatch time",
            send_queue_depth.clone(),
        );
        registry.register(
            "wss_mux_manifest_reloads",
            "Manifest reload attempts, labeled by result",
            manifest_reloads.clone(),
        );
        registry.register(
            "wss_mux_subscriptions_revoked",
            "Subscriptions dropped by a hot-reload re-validation, by reason",
            subscriptions_revoked.clone(),
        );
        registry.register(
            "wss_mux_frames_rate_limited",
            "Inbound client frames rejected by the per-connection rate limit",
            frames_rate_limited.clone(),
        );
        registry.register(
            "wss_mux_relay_sent",
            "Peer-relay deliveries that returned a success status",
            relay_sent.clone(),
        );
        registry.register(
            "wss_mux_relay_events_unwanted",
            "Relayed (peer-origin) events that matched no local subscription",
            relay_events_unwanted.clone(),
        );
        registry.register(
            "wss_mux_relay_flushes",
            "Relay coalescing flush cycles",
            relay_flushes.clone(),
        );
        registry.register(
            "wss_mux_relay_events_relayed",
            "Events handed to the peer-relay path",
            relay_events_relayed.clone(),
        );
        registry.register(
            "wss_mux_relay_queue_depth",
            "Current depth of the relay coalescing queue",
            relay_queue_depth.clone(),
        );
        registry.register(
            "wss_mux_relay_queue_dropped",
            "Relay batches dropped because the coalescing queue was full",
            relay_queue_dropped.clone(),
        );
        registry.register(
            "wss_mux_relay_flush_restarts",
            "Relay flush-task supervised restarts (should stay 0)",
            relay_flush_restarts.clone(),
        );
        registry.register(
            "wss_mux_relay_failed",
            "Peer-relay deliveries that failed, labeled by reason",
            relay_failed.clone(),
        );
        registry.register(
            "wss_mux_peers_known",
            "Peers currently in the discovered relay set",
            peers_known.clone(),
        );
        registry.register(
            "wss_mux_events_rejected",
            "Events rejected at ingest before dispatch, labeled by reason",
            events_rejected.clone(),
        );
        registry.register(
            "wss_mux_oidc_jwks_keys",
            "Keys currently in the cached OIDC JWKS",
            oidc_jwks_keys.clone(),
        );
        registry.register(
            "wss_mux_oidc_jwks_refresh",
            "OIDC JWKS refresh attempts, labeled by result",
            oidc_jwks_refresh.clone(),
        );

        Self {
            registry: Mutex::new(registry),
            connections_total,
            connections_active,
            subscriptions_active,
            events_dispatched,
            events_dropped,
            send_queue_depth,
            manifest_reloads,
            subscriptions_revoked,
            frames_rate_limited,
            relay_sent,
            relay_events_unwanted,
            relay_flushes,
            relay_events_relayed,
            relay_queue_depth,
            relay_queue_dropped,
            relay_flush_restarts,
            relay_failed,
            peers_known,
            events_rejected,
            oidc_jwks_keys,
            oidc_jwks_refresh,
        }
    }
}

impl Metrics {
    pub fn encode(&self) -> String {
        let mut buf = String::new();
        let registry = self.registry.lock().expect("metrics registry mutex");
        prometheus_client::encoding::text::encode(&mut buf, &registry).expect("encode metrics");
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalescing_metrics_are_registered() {
        let m = Metrics::default();
        m.relay_flushes.inc();
        m.relay_events_relayed.inc_by(3);
        m.relay_queue_depth.set(7);
        m.relay_queue_dropped.inc();
        m.relay_flush_restarts.inc();
        let body = m.encode();
        for needle in [
            "wss_mux_relay_flushes_total 1",
            "wss_mux_relay_events_relayed_total 3",
            "wss_mux_relay_queue_depth 7",
            "wss_mux_relay_queue_dropped_total 1",
            "wss_mux_relay_flush_restarts_total 1",
        ] {
            assert!(body.contains(needle), "missing {needle} in:\n{body}");
        }
    }
}
