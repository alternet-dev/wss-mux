use std::sync::Mutex;
use std::time::{Duration, Instant};

use dashmap::DashMap;

/// A monotonic-clock token bucket. `now` is passed in rather than read
/// internally so the refill behavior is deterministically testable.
///
/// The bucket starts full (a burst of `capacity` is allowed immediately),
/// refills at `refill_per_sec` tokens per second, and never exceeds
/// `capacity`. Each admitted frame costs one token.
#[derive(Debug)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_sec: u32, now: Instant) -> Self {
        let capacity = f64::from(capacity);
        Self {
            capacity,
            tokens: capacity,
            refill_per_sec: f64::from(refill_per_sec),
            last: now,
        }
    }

    /// Refill for the elapsed time, then try to spend one token. Returns
    /// `true` if the frame is admitted, `false` if it should be rejected.
    pub fn try_take(&mut self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.last = now;
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// A keyed pool of token buckets — one bucket per source identifier.
/// All buckets share the same `capacity` and `refill_per_sec`; the only
/// per-source state is the bucket's current token count and the last
/// time it was touched (for idle eviction).
///
/// Designed for the WS publish path, where the source key is the
/// connection's JWT `sub` (with a `conn:<id>` fallback). The same shape
/// fits the HTTP push path's per-source limit and can be lifted later.
///
/// `now` is injected per call to stay deterministically testable, the
/// same posture as `TokenBucket`.
#[derive(Debug)]
pub struct PerSourceRateLimiter {
    buckets: DashMap<String, Mutex<BucketEntry>>,
    capacity: u32,
    refill_per_sec: u32,
    idle_ttl: Duration,
}

#[derive(Debug)]
struct BucketEntry {
    bucket: TokenBucket,
    last_seen: Instant,
}

impl PerSourceRateLimiter {
    pub fn new(capacity: u32, refill_per_sec: u32, idle_ttl: Duration) -> Self {
        Self {
            buckets: DashMap::new(),
            capacity,
            refill_per_sec,
            idle_ttl,
        }
    }

    /// Get-or-insert the source's bucket, refill it for the elapsed time,
    /// and try to spend one token. Returns `true` if the publish is
    /// admitted, `false` if it should be rejected.
    ///
    /// `now` doubles as the source's last-seen timestamp for the idle
    /// eviction sweep.
    pub fn try_take(&self, source: &str, now: Instant) -> bool {
        // Two-step access pattern: a brief read lookup, then either
        // insert-on-miss or hold the bucket's Mutex over try_take. The
        // Mutex is held for the duration of refill + decrement only.
        if let Some(entry) = self.buckets.get(source) {
            let mut guard = entry.lock().expect("bucket mutex poisoned");
            guard.last_seen = now;
            return guard.bucket.try_take(now);
        }
        // Miss: insert a fresh entry. The full-burst allowance means the
        // first publish from a new source is always admitted (subject to
        // `capacity > 0`), then drains as expected.
        let entry = Mutex::new(BucketEntry {
            bucket: TokenBucket::new(self.capacity, self.refill_per_sec, now),
            last_seen: now,
        });
        // Race-safe: if another writer raced us, fall through to the
        // existing entry (with a fresh take against that one) so we
        // never double-admit by accident.
        match self.buckets.entry(source.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                let mut guard = occupied.get().lock().expect("bucket mutex poisoned");
                guard.last_seen = now;
                guard.bucket.try_take(now)
            }
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                let inserted = vacant.insert(entry);
                let mut guard = inserted.value().lock().expect("bucket mutex poisoned");
                guard.bucket.try_take(now)
            }
        }
    }

    /// Drop entries whose `last_seen` is older than `now - idle_ttl`.
    /// Returns the number of entries evicted. Intended for a low-
    /// frequency tokio interval task, not the publish hot path.
    pub fn sweep_idle(&self, now: Instant) -> usize {
        let cutoff = now.checked_sub(self.idle_ttl).unwrap_or(now);
        let before = self.buckets.len();
        self.buckets.retain(|_, entry| {
            // `retain`'s closure has `&mut Mutex`, so we own it exclusively
            // — no lock contention is possible. `get_mut` is infallible
            // for that reason; on a poisoned mutex we keep the entry
            // (conservative: it'll get re-evaluated on the next sweep).
            entry
                .get_mut()
                .map(|guard| guard.last_seen >= cutoff)
                .unwrap_or(true)
        });
        before - self.buckets.len()
    }

    /// Current count of tracked sources. Exposed as a gauge for
    /// capacity-planning visibility.
    pub fn active_sources(&self) -> usize {
        self.buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn starts_full_and_drains() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(3, 1, t0);
        // Three immediate takes succeed (full burst), the fourth fails.
        assert!(b.try_take(t0));
        assert!(b.try_take(t0));
        assert!(b.try_take(t0));
        assert!(!b.try_take(t0));
    }

    #[test]
    fn refills_over_time_up_to_capacity() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(2, 10, t0);
        assert!(b.try_take(t0));
        assert!(b.try_take(t0));
        assert!(!b.try_take(t0));

        // 10/sec refill → 100ms buys exactly one token.
        let t1 = t0 + Duration::from_millis(100);
        assert!(b.try_take(t1));
        assert!(!b.try_take(t1));

        // A long idle refills, but never past capacity (2).
        let t2 = t1 + Duration::from_secs(60);
        assert!(b.try_take(t2));
        assert!(b.try_take(t2));
        assert!(!b.try_take(t2));
    }

    #[test]
    fn zero_capacity_rejects_everything() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(0, 5, t0);
        assert!(!b.try_take(t0));
        assert!(!b.try_take(t0 + Duration::from_secs(1)));
    }

    #[test]
    fn per_source_isolates_distinct_sources() {
        let t0 = Instant::now();
        let l = PerSourceRateLimiter::new(2, 1, Duration::from_secs(60));
        // Source A drains.
        assert!(l.try_take("a", t0));
        assert!(l.try_take("a", t0));
        assert!(!l.try_take("a", t0));
        // Source B is unaffected — fresh full bucket.
        assert!(l.try_take("b", t0));
        assert!(l.try_take("b", t0));
        assert!(!l.try_take("b", t0));
    }

    #[test]
    fn per_source_shares_budget_for_same_key() {
        // Distinct connections that resolve to the same source key
        // (e.g. two WS sessions with the same JWT `sub`) share one
        // bucket. The pool's keying is the contract.
        let t0 = Instant::now();
        let l = PerSourceRateLimiter::new(2, 0, Duration::from_secs(60));
        assert!(l.try_take("alice", t0));
        assert!(l.try_take("alice", t0));
        assert!(
            !l.try_take("alice", t0),
            "second 'connection' for alice shares her budget"
        );
    }

    #[test]
    fn per_source_refills_over_time() {
        let t0 = Instant::now();
        let l = PerSourceRateLimiter::new(1, 10, Duration::from_secs(60));
        assert!(l.try_take("x", t0));
        assert!(!l.try_take("x", t0));
        // 10/sec refill → 100ms buys exactly one token.
        let t1 = t0 + Duration::from_millis(100);
        assert!(l.try_take("x", t1));
        assert!(!l.try_take("x", t1));
    }

    #[test]
    fn per_source_first_take_admits_new_source() {
        // A previously-unseen source should get the full burst —
        // landing on the limiter must not silently cost it a token it
        // didn't get to spend.
        let t0 = Instant::now();
        let l = PerSourceRateLimiter::new(3, 0, Duration::from_secs(60));
        assert!(l.try_take("fresh", t0));
        assert!(l.try_take("fresh", t0));
        assert!(l.try_take("fresh", t0));
        assert!(!l.try_take("fresh", t0));
    }

    #[test]
    fn per_source_sweep_evicts_idle_entries() {
        let t0 = Instant::now();
        let l = PerSourceRateLimiter::new(1, 0, Duration::from_secs(5));
        l.try_take("active", t0);
        l.try_take("idle", t0);
        assert_eq!(l.active_sources(), 2);

        // Touch "active" at t0+10s; "idle" stays at t0.
        let t1 = t0 + Duration::from_secs(10);
        l.try_take("active", t1);

        // Sweep at t1 with a 5s TTL → "idle" (last_seen at t0) is past
        // the cutoff; "active" (last_seen at t1) survives.
        let evicted = l.sweep_idle(t1);
        assert_eq!(evicted, 1);
        assert_eq!(l.active_sources(), 1);
    }

    #[test]
    fn per_source_sweep_at_origin_evicts_nothing() {
        // A sweep call at `now == last_seen` for every entry leaves
        // them alone — guards against off-by-one in the cutoff math.
        let t0 = Instant::now();
        let l = PerSourceRateLimiter::new(1, 0, Duration::from_secs(60));
        l.try_take("a", t0);
        l.try_take("b", t0);
        assert_eq!(l.sweep_idle(t0), 0);
        assert_eq!(l.active_sources(), 2);
    }

    #[test]
    fn per_source_zero_capacity_rejects_first_publish() {
        // A misconfigured (or intentionally-blocking) limiter rejects
        // the very first publish from every source — matching the
        // single-bucket behaviour.
        let t0 = Instant::now();
        let l = PerSourceRateLimiter::new(0, 5, Duration::from_secs(60));
        assert!(!l.try_take("x", t0));
        assert!(!l.try_take("x", t0 + Duration::from_secs(1)));
        // Different source, same outcome.
        assert!(!l.try_take("y", t0));
    }
}
