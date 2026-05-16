use std::time::Instant;

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
}
