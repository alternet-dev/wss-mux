//! Minimal `/metrics` scraper — parses just the `wss_mux_*` values the
//! scenarios report against, out of the OpenMetrics text exposition.

use anyhow::{Context, Result};

/// The subset of `wss_mux_*` metrics the harness reads.
#[derive(Debug, Default)]
pub struct MetricsSnapshot {
    pub connections_active: f64,
    pub subscriptions_active: f64,
    /// Sum across every `{stream=…}` series. The dispatcher counts one
    /// per-subscription delivery, so one push fanned to N subscribers
    /// adds N here.
    pub events_dispatched: f64,
    /// Sum across every `{reason=…}` series.
    pub events_dropped: f64,
}

/// The movement of the cumulative counters between two snapshots.
#[derive(Debug)]
pub struct MetricsDelta {
    pub events_dispatched: f64,
    pub events_dropped: f64,
}

impl MetricsSnapshot {
    /// Counter movement from `before` to `self`.
    pub fn delta(&self, before: &MetricsSnapshot) -> MetricsDelta {
        MetricsDelta {
            events_dispatched: self.events_dispatched - before.events_dispatched,
            events_dropped: self.events_dropped - before.events_dropped,
        }
    }
}

/// GET `<base_url>/metrics` and parse the snapshot.
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

fn parse(body: &str) -> MetricsSnapshot {
    MetricsSnapshot {
        connections_active: sum_series(body, "wss_mux_connections_active"),
        subscriptions_active: sum_series(body, "wss_mux_subscriptions_active"),
        events_dispatched: sum_series(body, "wss_mux_events_dispatched_total"),
        events_dropped: sum_series(body, "wss_mux_events_dropped_total"),
    }
}

/// Sum the value of every sample line belonging to metric `name`, across
/// all label sets. A sample line is `name <v>` or `name{labels} <v>`;
/// the delimiter check after the name stops a metric whose name merely
/// has `name` as a prefix from being summed in too.
fn sum_series(body: &str, name: &str) -> f64 {
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
# EOF
";

    #[test]
    fn parses_gauges_and_sums_labeled_counters() {
        let snap = parse(SAMPLE);
        assert_eq!(snap.connections_active, 12.0);
        assert_eq!(snap.subscriptions_active, 12.0);
        assert_eq!(snap.events_dispatched, 500.0);
        assert_eq!(snap.events_dropped, 10.0);
    }

    #[test]
    fn absent_metric_sums_to_zero() {
        assert_eq!(sum_series("# EOF\n", "wss_mux_events_dropped_total"), 0.0);
    }

    #[test]
    fn a_name_prefix_does_not_cross_match() {
        // A query for `wss_mux_relay_sent` must not absorb the value of
        // `wss_mux_relay_sent_total` — the delimiter check guards this.
        let body = "wss_mux_relay_sent 1\nwss_mux_relay_sent_total 9\n";
        assert_eq!(sum_series(body, "wss_mux_relay_sent"), 1.0);
        assert_eq!(sum_series(body, "wss_mux_relay_sent_total"), 9.0);
    }

    #[test]
    fn delta_subtracts_counters() {
        let before = MetricsSnapshot {
            events_dispatched: 100.0,
            events_dropped: 2.0,
            ..Default::default()
        };
        let after = MetricsSnapshot {
            events_dispatched: 350.0,
            events_dropped: 5.0,
            ..Default::default()
        };
        let d = after.delta(&before);
        assert_eq!(d.events_dispatched, 250.0);
        assert_eq!(d.events_dropped, 3.0);
    }
}
