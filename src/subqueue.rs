//! Per-subscription in-flight accounting.
//!
//! v0.4 makes the send-queue/overflow unit the *subscription* rather
//! than the connection. Rather than give every subscription its own
//! channel (and rebuild the writer around a dynamic merge), the single
//! per-connection channel is kept and each subscription's outstanding
//! frame count is tracked here. The dispatcher [`try_reserve`]s a slot
//! against the subscription's effective cap before enqueuing; the
//! writer [`release`]s once the frame has been written. A subscription
//! that cannot reserve has overflowed and is dropped on its own — the
//! connection and its other subscriptions are unaffected.
//!
//! [`try_reserve`]: SubQueues::try_reserve
//! [`release`]: SubQueues::release

use dashmap::DashMap;

use crate::registry::{ConnId, SubId};

/// In-flight frame counts keyed by `(connection, subscription)`.
#[derive(Default)]
pub struct SubQueues {
    counts: DashMap<(ConnId, SubId), usize>,
}

impl SubQueues {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserve one in-flight slot for `(conn_id, sub_id)` if it is
    /// currently below `cap`. Returns `true` on success (count
    /// incremented), `false` if the subscription is already at `cap`
    /// (the caller treats this as overflow for that subscription).
    pub fn try_reserve(&self, conn_id: ConnId, sub_id: &str, cap: usize) -> bool {
        let mut count = self
            .counts
            .entry((conn_id, sub_id.to_string()))
            .or_insert(0);
        if *count >= cap {
            return false;
        }
        *count += 1;
        true
    }

    /// Release one in-flight slot (the writer has written the frame).
    /// Saturating: a release with no outstanding count is a no-op. The
    /// entry is removed at zero to bound memory.
    pub fn release(&self, conn_id: ConnId, sub_id: &str) {
        let key = (conn_id, sub_id.to_string());
        if let Some(mut count) = self.counts.get_mut(&key) {
            *count = count.saturating_sub(1);
        } else {
            return;
        }
        // Guard dropped above; `remove_if` re-checks the value
        // atomically under the shard lock, so a concurrent reserve
        // that resurrected the count is not deleted.
        self.counts.remove_if(&key, |_, &count| count == 0);
    }

    /// Forget the subscription entirely (unsubscribe / revoke /
    /// overflow drop / connection teardown). Any outstanding count is
    /// discarded — frames still sitting in the shared channel are
    /// dropped without a matching release, which `drop_sub` accounts
    /// for by removing the whole entry.
    pub fn drop_sub(&self, conn_id: ConnId, sub_id: &str) {
        self.counts.remove(&(conn_id, sub_id.to_string()));
    }

    /// Current in-flight count for `(conn_id, sub_id)` (`0` if absent).
    pub fn in_flight(&self, conn_id: ConnId, sub_id: &str) -> usize {
        self.counts
            .get(&(conn_id, sub_id.to_string()))
            .map(|c| *c)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn reserve_succeeds_below_cap_and_fails_at_cap() {
        let q = SubQueues::new();
        assert!(q.try_reserve(1, "s1", 2));
        assert!(q.try_reserve(1, "s1", 2));
        assert!(
            !q.try_reserve(1, "s1", 2),
            "third reserve must fail at cap 2"
        );
        assert_eq!(q.in_flight(1, "s1"), 2);
    }

    #[test]
    fn release_frees_a_slot() {
        let q = SubQueues::new();
        assert!(q.try_reserve(1, "s1", 1));
        assert!(!q.try_reserve(1, "s1", 1));
        q.release(1, "s1");
        assert!(
            q.try_reserve(1, "s1", 1),
            "a freed slot must be reservable again"
        );
        assert_eq!(q.in_flight(1, "s1"), 1);
    }

    #[test]
    fn drop_sub_clears_all_in_flight() {
        let q = SubQueues::new();
        q.try_reserve(1, "s1", 8);
        q.try_reserve(1, "s1", 8);
        q.drop_sub(1, "s1");
        assert_eq!(q.in_flight(1, "s1"), 0);
        // Reservable from scratch afterwards.
        assert!(q.try_reserve(1, "s1", 1));
    }

    #[test]
    fn in_flight_zero_for_unknown() {
        let q = SubQueues::new();
        assert_eq!(q.in_flight(42, "nope"), 0);
    }

    #[test]
    fn release_unknown_is_noop() {
        let q = SubQueues::new();
        q.release(1, "s1"); // must not panic / underflow
        assert_eq!(q.in_flight(1, "s1"), 0);
    }

    #[test]
    fn independent_subs_have_independent_counts() {
        let q = SubQueues::new();
        // Same connection, two subscriptions: filling one must not
        // block the other.
        assert!(q.try_reserve(1, "s1", 1));
        assert!(!q.try_reserve(1, "s1", 1));
        assert!(q.try_reserve(1, "s2", 1), "s2 is independent of s1");
        // And different connections are independent too.
        assert!(q.try_reserve(2, "s1", 1));
    }

    #[test]
    fn concurrent_reserve_release_conserves_count() {
        let q = Arc::new(SubQueues::new());
        let threads: Vec<_> = (0..16)
            .map(|_| {
                let q = Arc::clone(&q);
                thread::spawn(move || {
                    for _ in 0..256 {
                        if q.try_reserve(1, "s1", usize::MAX) {
                            q.release(1, "s1");
                        }
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        assert_eq!(
            q.in_flight(1, "s1"),
            0,
            "every reserve was released; net in-flight must be zero"
        );
    }
}
