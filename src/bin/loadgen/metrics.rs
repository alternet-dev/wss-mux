//! Minimal `/metrics` scraper — parses just the `wss_mux_*` values the
//! scenarios report against, out of the OpenMetrics text exposition.

use anyhow::{Context, Result};

/// The subset of `wss_mux_*` metrics the harness reads. Counter fields
/// are cumulative; gauge fields are point-in-time.
#[derive(Debug, Default)]
pub struct MetricsSnapshot {
    pub connections_active: f64,
    pub subscriptions_active: f64,
    /// `wss_mux_events_dispatched_total`, summed across `{stream=…}`.
    /// One per-subscription delivery, so a push fanned to N subscribers
    /// adds N.
    pub events_dispatched: f64,
    /// `wss_mux_events_dropped_total{reason="overflow"}` — a full
    /// per-subscription channel. The benign `no_subscribers` reason is
    /// excluded: it dominates on a non-subscriber fleet instance and is
    /// captured instead by `relay_events_unwanted`.
    pub overflow_drops: f64,
    /// `wss_mux_relay_events_relayed_total` — events handed to the
    /// peer-relay path (counted on the relaying instance).
    pub relay_events_relayed: f64,
    /// `wss_mux_relay_sent_total` — successful per-peer relay POSTs.
    pub relay_sent: f64,
    /// `wss_mux_relay_events_unwanted_total` — relayed events that
    /// matched no local subscription (the cross-instance-waste signal).
    pub relay_events_unwanted: f64,
}

impl MetricsSnapshot {
    /// Add another snapshot in place — used to sum a fleet's instances
    /// into one fleet-wide total.
    fn accumulate(&mut self, other: &MetricsSnapshot) {
        self.connections_active += other.connections_active;
        self.subscriptions_active += other.subscriptions_active;
        self.events_dispatched += other.events_dispatched;
        self.overflow_drops += other.overflow_drops;
        self.relay_events_relayed += other.relay_events_relayed;
        self.relay_sent += other.relay_sent;
        self.relay_events_unwanted += other.relay_events_unwanted;
    }

    /// Counter movement from `before` to `self`.
    pub fn delta(&self, before: &MetricsSnapshot) -> MetricsDelta {
        MetricsDelta {
            events_dispatched: self.events_dispatched - before.events_dispatched,
            overflow_drops: self.overflow_drops - before.overflow_drops,
            relay_events_relayed: self.relay_events_relayed - before.relay_events_relayed,
            relay_sent: self.relay_sent - before.relay_sent,
            relay_events_unwanted: self.relay_events_unwanted - before.relay_events_unwanted,
        }
    }
}

/// Counter movement between two snapshots.
#[derive(Debug)]
pub struct MetricsDelta {
    pub events_dispatched: f64,
    pub overflow_drops: f64,
    pub relay_events_relayed: f64,
    pub relay_sent: f64,
    pub relay_events_unwanted: f64,
}

/// GET `<base_url>/metrics` and parse one instance's snapshot.
pub async fn scrape(http: &reqwest::Client, base_url: &str) -> Result<MetricsSnapshot> {
    let body = http
        .get(format!("{base_url}/metrics"))
        .send()
        .await
        .context("GET /metrics")?
        .text()
        .await
        .context("read /metrics body")?;
    Ok(parse(&body))
}

/// Scrape every instance in `base_urls` and sum into one fleet-wide
/// snapshot: counter totals add, and so do the active-connection /
/// subscription gauges (a fleet-wide live count).
pub async fn scrape_fleet(http: &reqwest::Client, base_urls: &[&str]) -> Result<MetricsSnapshot> {
    let mut total = MetricsSnapshot::default();
    for &base in base_urls {
        total.accumulate(&scrape(http, base).await?);
    }
    Ok(total)
}

fn parse(body: &str) -> MetricsSnapshot {
    MetricsSnapshot {
        connections_active: sum_series(body, "wss_mux_connections_active", None),
        subscriptions_active: sum_series(body, "wss_mux_subscriptions_active", None),
        events_dispatched: sum_series(body, "wss_mux_events_dispatched_total", None),
        overflow_drops: sum_series(
            body,
            "wss_mux_events_dropped_total",
            Some("reason=\"overflow\""),
        ),
        relay_events_relayed: sum_series(body, "wss_mux_relay_events_relayed_total", None),
        relay_sent: sum_series(body, "wss_mux_relay_sent_total", None),
        relay_events_unwanted: sum_series(body, "wss_mux_relay_events_unwanted_total", None),
    }
}

/// Sum the value of every sample line belonging to metric `name`. With
/// `label` set, only lines whose label block contains that substring
/// count (e.g. `reason="overflow"`); with `None`, every label set is
/// summed. The delimiter check after the name stops a metric whose name
/// merely has `name` as a prefix from matching.
fn sum_series(body: &str, name: &str, label: Option<&str>) -> f64 {
    let mut total = 0.0;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(rest) = line.strip_prefix(name) else {
            continue;
        };
        if !matches!(rest.chars().next(), Some(' ') | Some('{')) {
            continue;
        }
        if let Some(needle) = label {
            if !rest.contains(needle) {
                continue;
            }
        }
        if let Some(value) = line
            .split_whitespace()
            .next_back()
            .and_then(|token| token.parse::<f64>().ok())
        {
            total += value;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# HELP wss_mux_connections_active Currently open WebSocket connections
# TYPE wss_mux_connections_active gauge
wss_mux_connections_active 12
# TYPE wss_mux_subscriptions_active gauge
wss_mux_subscriptions_active 12
# TYPE wss_mux_events_dispatched counter
wss_mux_events_dispatched_total{stream=\"loadgen\"} 400
wss_mux_events_dispatched_total{stream=\"other\"} 100
# TYPE wss_mux_events_dropped counter
wss_mux_events_dropped_total{reason=\"overflow\"} 3
wss_mux_events_dropped_total{reason=\"no_subscribers\"} 7
# TYPE wss_mux_relay_events_relayed counter
wss_mux_relay_events_relayed_total 50
# TYPE wss_mux_relay_sent counter
wss_mux_relay_sent_total 150
# TYPE wss_mux_relay_events_unwanted counter
wss_mux_relay_events_unwanted_total 100
# EOF
";

    #[test]
    fn parses_gauges_relay_counters_and_overflow_only_drops() {
        let snap = parse(SAMPLE);
        assert_eq!(snap.connections_active, 12.0);
        assert_eq!(snap.subscriptions_active, 12.0);
        assert_eq!(snap.events_dispatched, 500.0);
        // Only the overflow reason — the no_subscribers 7 is excluded.
        assert_eq!(snap.overflow_drops, 3.0);
        assert_eq!(snap.relay_events_relayed, 50.0);
        assert_eq!(snap.relay_sent, 150.0);
        assert_eq!(snap.relay_events_unwanted, 100.0);
    }

    #[test]
    fn label_filter_selects_one_series() {
        let body = "wss_mux_events_dropped_total{reason=\"overflow\"} 3\n\
                    wss_mux_events_dropped_total{reason=\"no_subscribers\"} 7\n";
        assert_eq!(
            sum_series(
                body,
                "wss_mux_events_dropped_total",
                Some("reason=\"overflow\"")
            ),
            3.0
        );
        assert_eq!(sum_series(body, "wss_mux_events_dropped_total", None), 10.0);
    }

    #[test]
    fn absent_metric_sums_to_zero() {
        assert_eq!(
            sum_series("# EOF\n", "wss_mux_events_dropped_total", None),
            0.0
        );
    }

    #[test]
    fn a_name_prefix_does_not_cross_match() {
        // A query for `wss_mux_relay_sent` must not absorb the value of
        // `wss_mux_relay_sent_total` — the delimiter check guards this.
        let body = "wss_mux_relay_sent 1\nwss_mux_relay_sent_total 9\n";
        assert_eq!(sum_series(body, "wss_mux_relay_sent", None), 1.0);
        assert_eq!(sum_series(body, "wss_mux_relay_sent_total", None), 9.0);
    }

    #[test]
    fn accumulate_sums_fleet_instances() {
        let mut total = MetricsSnapshot::default();
        total.accumulate(&parse(SAMPLE));
        total.accumulate(&parse(SAMPLE));
        assert_eq!(total.events_dispatched, 1000.0);
        assert_eq!(total.relay_events_unwanted, 200.0);
    }

    #[test]
    fn delta_subtracts_counters() {
        let before = MetricsSnapshot {
            events_dispatched: 100.0,
            relay_events_relayed: 10.0,
            ..Default::default()
        };
        let after = MetricsSnapshot {
            events_dispatched: 350.0,
            relay_events_relayed: 40.0,
            ..Default::default()
        };
        let d = after.delta(&before);
        assert_eq!(d.events_dispatched, 250.0);
        assert_eq!(d.relay_events_relayed, 30.0);
    }
}
