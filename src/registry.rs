use dashmap::DashMap;
use std::collections::BTreeSet;

pub type ConnId = u64;
pub type SubId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub conn_id: ConnId,
    pub sub_id: SubId,
    pub key: Option<String>,
}

/// Aggregate interest for one stream. `all_keys` is true when a
/// keyless subscription exists or the exact-key set crossed the
/// caller's bound and must fail open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryInterest {
    pub all_keys: bool,
    pub keys: Vec<String>,
}

#[derive(Default)]
pub struct Registry {
    streams: DashMap<String, Vec<Binding>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, stream: &str, conn_id: ConnId, sub_id: SubId, key: Option<String>) {
        self.streams
            .entry(stream.to_string())
            .or_default()
            .push(Binding {
                conn_id,
                sub_id,
                key,
            });
    }

    pub fn unsubscribe(&self, stream: &str, conn_id: ConnId, sub_id: &str) {
        if let Some(mut entry) = self.streams.get_mut(stream) {
            entry.retain(|b| !(b.conn_id == conn_id && b.sub_id == sub_id));
        }
    }

    pub fn matches(&self, stream: &str, event_key: Option<&str>) -> Vec<(ConnId, SubId)> {
        let Some(bindings) = self.streams.get(stream) else {
            return Vec::new();
        };
        bindings
            .iter()
            .filter(|b| b.key.is_none() || b.key.as_deref() == event_key)
            .map(|b| (b.conn_id, b.sub_id.clone()))
            .collect()
    }

    pub fn binding_count(&self, stream: &str) -> usize {
        self.streams.get(stream).map(|e| e.len()).unwrap_or(0)
    }

    /// Summarize local interest without exposing connection or
    /// subscription identities. Exact keys are sorted and deduplicated.
    /// Crossing `max_keys` collapses to `all_keys`, which can create
    /// extra relay traffic but cannot suppress a wanted event.
    pub fn interest(&self, stream: &str, max_keys: usize) -> Option<RegistryInterest> {
        let bindings = self.streams.get(stream)?;
        if bindings.is_empty() {
            return None;
        }
        if bindings.iter().any(|binding| binding.key.is_none()) {
            return Some(RegistryInterest {
                all_keys: true,
                keys: Vec::new(),
            });
        }

        let mut keys = BTreeSet::new();
        for key in bindings.iter().filter_map(|binding| binding.key.as_ref()) {
            keys.insert(key.clone());
            if keys.len() > max_keys {
                return Some(RegistryInterest {
                    all_keys: true,
                    keys: Vec::new(),
                });
            }
        }
        Some(RegistryInterest {
            all_keys: false,
            keys: keys.into_iter().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn subscribe_then_match_returns_binding() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), None);
        let m = r.matches("chat_messages", None);
        assert_eq!(m, vec![(1u64, "s1".to_string())]);
    }

    #[test]
    fn match_on_empty_stream_returns_empty() {
        let r = Registry::new();
        assert_eq!(r.matches("chat_messages", None), Vec::new());
    }

    #[test]
    fn multiple_subs_same_stream_all_match() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), None);
        r.subscribe("chat_messages", 2, "s2".into(), None);
        let m = r.matches("chat_messages", None);
        assert_eq!(m.len(), 2);
        assert!(m.contains(&(1u64, "s1".to_string())));
        assert!(m.contains(&(2u64, "s2".to_string())));
    }

    #[test]
    fn keyless_sub_matches_keyed_event() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), None);
        let m = r.matches("chat_messages", Some("room-42"));
        assert_eq!(m, vec![(1u64, "s1".to_string())]);
    }

    #[test]
    fn keyed_sub_matches_same_key_event() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), Some("room-42".into()));
        let m = r.matches("chat_messages", Some("room-42"));
        assert_eq!(m, vec![(1u64, "s1".to_string())]);
    }

    #[test]
    fn keyed_sub_does_not_match_different_key() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), Some("room-42".into()));
        let m = r.matches("chat_messages", Some("room-43"));
        assert!(m.is_empty());
    }

    #[test]
    fn keyed_sub_does_not_match_keyless_event() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), Some("room-42".into()));
        let m = r.matches("chat_messages", None);
        assert!(m.is_empty());
    }

    #[test]
    fn unsubscribe_removes_binding() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), None);
        r.unsubscribe("chat_messages", 1, "s1");
        assert!(r.matches("chat_messages", None).is_empty());
    }

    #[test]
    fn unsubscribe_only_removes_matching_binding() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), None);
        r.subscribe("chat_messages", 2, "s2".into(), None);
        r.unsubscribe("chat_messages", 1, "s1");
        let m = r.matches("chat_messages", None);
        assert_eq!(m, vec![(2u64, "s2".to_string())]);
    }

    #[test]
    fn unsubscribe_unknown_is_noop() {
        let r = Registry::new();
        r.unsubscribe("chat_messages", 1, "s1");
        assert!(r.matches("chat_messages", None).is_empty());
    }

    #[test]
    fn concurrent_subscribe_does_not_deadlock_and_records_all() {
        let r = Arc::new(Registry::new());
        let threads: Vec<_> = (0..16)
            .map(|i| {
                let r = Arc::clone(&r);
                thread::spawn(move || {
                    for j in 0..32 {
                        r.subscribe("chat_messages", i as u64, format!("s{j}"), None);
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        assert_eq!(r.binding_count("chat_messages"), 16 * 32);
    }

    #[test]
    fn interest_is_sorted_deduplicated_and_key_specific() {
        let r = Registry::new();
        r.subscribe("chat_messages", 1, "s1".into(), Some("room-b".into()));
        r.subscribe("chat_messages", 2, "s2".into(), Some("room-a".into()));
        r.subscribe("chat_messages", 3, "s3".into(), Some("room-b".into()));
        assert_eq!(
            r.interest("chat_messages", 10),
            Some(RegistryInterest {
                all_keys: false,
                keys: vec!["room-a".into(), "room-b".into()],
            })
        );
    }

    #[test]
    fn keyless_or_over_bound_interest_fails_open_to_all_keys() {
        let r = Registry::new();
        r.subscribe("keyless", 1, "s1".into(), None);
        assert_eq!(
            r.interest("keyless", 1),
            Some(RegistryInterest {
                all_keys: true,
                keys: vec![],
            })
        );

        r.subscribe("many", 1, "s1".into(), Some("a".into()));
        r.subscribe("many", 2, "s2".into(), Some("b".into()));
        assert!(r.interest("many", 1).unwrap().all_keys);
        assert!(r.interest("missing", 1).is_none());
    }
}
