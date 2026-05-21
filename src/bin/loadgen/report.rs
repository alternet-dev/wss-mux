//! Result types: `Serialize` for `--json`, `Display` for the human
//! report. Every result carries a `RunMeta` header so a number is never
//! read without its mode and window.

use std::fmt;

use serde::Serialize;

/// Header stamped onto every result.
#[derive(Debug, Serialize)]
pub struct RunMeta {
    pub scenario: String,
    /// `in-process` or `external`.
    pub mode: String,
    pub duration_secs: u64,
}

/// p50/p90/p99/max over a set of millisecond samples.
#[derive(Debug, Serialize)]
pub struct Percentiles {
    pub p50: f64,
    pub p90: f64,
    pub p99: f64,
    pub max: f64,
}

impl Percentiles {
    /// Nearest-rank percentiles over `samples` (milliseconds). Empty
    /// input yields all-zero — nothing was measured.
    pub fn from_millis(mut samples: Vec<f64>) -> Self {
        if samples.is_empty() {
            return Self {
                p50: 0.0,
                p90: 0.0,
                p99: 0.0,
                max: 0.0,
            };
        }
        samples.sort_by(f64::total_cmp);
        let at = |quantile: f64| -> f64 {
            let rank = (quantile * samples.len() as f64).ceil() as usize;
            samples[rank.saturating_sub(1).min(samples.len() - 1)]
        };
        Self {
            p50: at(0.50),
            p90: at(0.90),
            p99: at(0.99),
            max: *samples.last().expect("samples is non-empty here"),
        }
    }
}

impl fmt::Display for Percentiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "p50={:.3}  p90={:.3}  p99={:.3}  max={:.3}",
            self.p50, self.p90, self.p99, self.max
        )
    }
}

#[derive(Debug, Serialize)]
pub struct ThroughputReport {
    pub meta: RunMeta,
    pub subscribers: usize,
    pub producers: usize,
    pub pushes_ok: u64,
    pub pushes_failed: u64,
    pub push_rate_per_sec: f64,
    /// `wss_mux_events_dispatched_total` delta — per-subscription
    /// deliveries, so ≈ `pushes_ok × subscribers` when nothing drops.
    pub events_delivered: u64,
    pub delivery_rate_per_sec: f64,
    pub events_dropped: u64,
    pub push_latency_ms: Percentiles,
}

#[derive(Debug, Serialize)]
pub struct LatencyReport {
    pub meta: RunMeta,
    pub subscribers: usize,
    pub interval_ms: u64,
    pub samples: usize,
    pub delivery_latency_ms: Percentiles,
}

#[derive(Debug, Serialize)]
pub struct ConnectionsReport {
    pub meta: RunMeta,
    pub requested: usize,
    pub connected: usize,
    pub connections_active: i64,
    pub subscriptions_active: i64,
    /// Resident set size, KiB. `Some` only in in-process mode, where it
    /// covers harness + server in one process; `None` externally.
    pub rss_kib: Option<u64>,
}

/// One scenario's result. `untagged` so `--json` emits the bare report
/// object; the `meta.scenario` field identifies which scenario it is.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Report {
    Throughput(ThroughputReport),
    Latency(LatencyReport),
    Connections(ConnectionsReport),
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Report::Throughput(r) => r.fmt(f),
            Report::Latency(r) => r.fmt(f),
            Report::Connections(r) => r.fmt(f),
        }
    }
}

impl fmt::Display for ThroughputReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "throughput  (mode: {}, {}s, {} subscribers, {} producers)",
            self.meta.mode, self.meta.duration_secs, self.subscribers, self.producers
        )?;
        writeln!(
            f,
            "  pushes:            ok={}  failed={}  ({:.0}/s ingest ceiling)",
            self.pushes_ok, self.pushes_failed, self.push_rate_per_sec
        )?;
        writeln!(
            f,
            "  deliveries:        {}  ({:.0}/s fan-out)",
            self.events_delivered, self.delivery_rate_per_sec
        )?;
        writeln!(f, "  events dropped:    {}", self.events_dropped)?;
        write!(f, "  push latency (ms): {}", self.push_latency_ms)
    }
}

impl fmt::Display for LatencyReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "latency  (mode: {}, {}s, {} subscribers, {}ms interval)",
            self.meta.mode, self.meta.duration_secs, self.subscribers, self.interval_ms
        )?;
        writeln!(f, "  samples:               {}", self.samples)?;
        write!(f, "  delivery latency (ms): {}", self.delivery_latency_ms)
    }
}

impl fmt::Display for ConnectionsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "connections  (mode: {}, {}s)",
            self.meta.mode, self.meta.duration_secs
        )?;
        writeln!(
            f,
            "  connections:   requested={}  connected={}  active={}",
            self.requested, self.connected, self.connections_active
        )?;
        writeln!(f, "  subscriptions: active={}", self.subscriptions_active)?;
        match self.rss_kib {
            Some(kib) => write!(f, "  rss:           {kib} KiB (harness + server)"),
            None => write!(f, "  rss:           n/a (external mode)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_of_empty_input_are_zero() {
        let p = Percentiles::from_millis(Vec::new());
        assert_eq!((p.p50, p.p90, p.p99, p.max), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn percentiles_use_nearest_rank() {
        // 1..=100: nearest-rank q lands on sample ceil(q*n).
        let p = Percentiles::from_millis((1..=100).map(f64::from).collect());
        assert_eq!(p.p50, 50.0);
        assert_eq!(p.p90, 90.0);
        assert_eq!(p.p99, 99.0);
        assert_eq!(p.max, 100.0);
    }

    #[test]
    fn percentiles_of_a_single_sample() {
        let p = Percentiles::from_millis(vec![7.5]);
        assert_eq!((p.p50, p.p90, p.p99, p.max), (7.5, 7.5, 7.5, 7.5));
    }
}
