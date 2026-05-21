//! Named traffic topologies. Each wraps a fixed subscriber-placement
//! and key-cardinality preset, so a fleet run is reproducible and the
//! resulting relay-waste numbers are directly comparable across runs.

use clap::ValueEnum;

/// Target subscribers per key in the `keyed` topology — a small "room".
const KEYED_AUDIENCE: usize = 4;

/// How subscribers and keys are arranged across the fleet — the
/// variable that drives cross-instance relay waste.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Topology {
    /// One unkeyed stream, subscribers balanced across every instance —
    /// a popular feed behind a healthy load balancer. Every instance
    /// has subscribers, so every relay is wanted: waste stays ≈ 0.
    Broadcast,
    /// One unkeyed stream, all subscribers on a single instance — a
    /// misrouting load balancer, or an audience too small to spread.
    /// The worst case for one stream.
    Concentrated,
    /// Many keys, each a small audience, subscribers scattered across
    /// the fleet — chat rooms or per-user channels. A keyed event only
    /// ever interests a few instances, so relay waste is intrinsic and
    /// climbs with fleet size; no load balancer can remove it.
    Keyed,
}

/// Where one subscriber sits: which fleet instance it connects to, and
/// the key it subscribes with (`None` ⇒ unkeyed).
pub struct Placement {
    pub instance: usize,
    pub key: Option<String>,
}

impl Topology {
    /// Lower-case name, for the result header.
    pub fn as_str(self) -> &'static str {
        match self {
            Topology::Broadcast => "broadcast",
            Topology::Concentrated => "concentrated",
            Topology::Keyed => "keyed",
        }
    }

    /// Distinct key count (`1` ⇒ unkeyed).
    fn key_count(self, subscribers: usize) -> usize {
        match self {
            Topology::Keyed => subscribers.div_ceil(KEYED_AUDIENCE).max(1),
            Topology::Broadcast | Topology::Concentrated => 1,
        }
    }

    /// Placement for every subscriber. `fleet` is the instance count;
    /// `concentrated_idx` is the lone instance the `concentrated`
    /// topology pins every subscriber to.
    pub fn subscriber_plan(
        self,
        subscribers: usize,
        fleet: usize,
        concentrated_idx: usize,
    ) -> Vec<Placement> {
        let fleet = fleet.max(1);
        let keys = self.key_count(subscribers);
        (0..subscribers)
            .map(|i| {
                let instance = match self {
                    Topology::Concentrated => concentrated_idx.min(fleet - 1),
                    Topology::Broadcast => i % fleet,
                    // Scatter `keyed` subscribers so a key's audience
                    // lands on instances uncorrelated with the key.
                    Topology::Keyed => (scatter(i as u64) % fleet as u64) as usize,
                };
                let key = match self {
                    Topology::Keyed => Some(format!("room-{}", i % keys)),
                    Topology::Broadcast | Topology::Concentrated => None,
                };
                Placement { instance, key }
            })
            .collect()
    }

    /// The keys the producer cycles through. Unkeyed topologies yield a
    /// single `None`; `keyed` yields one entry per room.
    pub fn producer_keys(self, subscribers: usize) -> Vec<Option<String>> {
        match self {
            Topology::Keyed => (0..self.key_count(subscribers))
                .map(|k| Some(format!("room-{k}")))
                .collect(),
            Topology::Broadcast | Topology::Concentrated => vec![None],
        }
    }
}

/// SplitMix64 finalizer — a fast deterministic hash used to scatter
/// `keyed` subscribers across instances without a `rand` dependency.
fn scatter(i: u64) -> u64 {
    let mut z = i.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_spreads_subscribers_evenly_and_unkeyed() {
        let plan = Topology::Broadcast.subscriber_plan(8, 4, 1);
        for inst in 0..4 {
            let on_inst = plan.iter().filter(|p| p.instance == inst).count();
            assert_eq!(on_inst, 2, "even spread across the fleet");
        }
        assert!(plan.iter().all(|p| p.key.is_none()), "broadcast is unkeyed");
    }

    #[test]
    fn concentrated_pins_every_subscriber_to_one_instance() {
        let plan = Topology::Concentrated.subscriber_plan(20, 4, 1);
        assert!(plan.iter().all(|p| p.instance == 1));
        assert!(plan.iter().all(|p| p.key.is_none()));
    }

    #[test]
    fn keyed_uses_small_rooms_the_producer_also_pushes() {
        let subscribers = 40;
        let plan = Topology::Keyed.subscriber_plan(subscribers, 8, 1);
        let keys = Topology::Keyed.producer_keys(subscribers);
        // 40 / 4 ⇒ 10 rooms.
        assert_eq!(keys.len(), 10);
        // Every subscriber is keyed to a room the producer pushes.
        for p in &plan {
            let key = p.key.as_deref().expect("keyed subscriber has a key");
            assert!(keys.iter().any(|k| k.as_deref() == Some(key)));
        }
        // The scatter spreads a keyed fleet over more than one instance.
        let distinct = plan
            .iter()
            .map(|p| p.instance)
            .collect::<std::collections::HashSet<_>>();
        assert!(
            distinct.len() > 1,
            "keyed subscribers scatter across instances"
        );
    }

    #[test]
    fn unkeyed_topologies_have_a_single_none_producer_key() {
        assert_eq!(Topology::Broadcast.producer_keys(50), vec![None]);
        assert_eq!(Topology::Concentrated.producer_keys(50), vec![None]);
    }
}
