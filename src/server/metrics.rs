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
