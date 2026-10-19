//! Outbound peer-relay: the relay wire batch, the per-push relay
//! decision, and (v0.5) the coalescing flush worker.

use axum::http::header;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::envelope::EventEnvelope;
use crate::peers::PeerUrl;
use crate::registry::Registry;
use crate::server::metrics::{RelayFailure, RelayFailureLabel, ReloadResult, ReloadResultLabel};
use crate::server::AppState;

/// Maximum exact keys advertised for one stream. Above this bound the
/// peer advertises `all_keys`, preserving delivery while bounding each
/// cached snapshot.
pub const MAX_INTEREST_KEYS_PER_STREAM: usize = 4096;

/// Internal relay body: a batch of already-canonical envelopes,
/// CBOR-encoded. Independent of the producer-facing `WSS_MUX_ENVELOPE_*`
/// config — peers exchange canonical envelopes, so receiving a relay
/// needs no envelope-path coupling. A single producer event relays as a
/// one-element batch so relay code stays uniform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelayBatch {
    pub events: Vec<EventEnvelope>,
}

/// Streams for which a sender wants aggregate peer interest. Keeping
/// this request explicit avoids advertising unused non-selective
/// streams and stays safe across mixed-manifest rolling deploys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterestRequest {
    pub streams: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamInterest {
    pub stream: String,
    pub all_keys: bool,
    pub keys: Vec<String>,
}

/// Aggregate, identity-free subscription interest advertised by one
/// peer. An absent stream means that peer has no matching subscriber.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterestSnapshot {
    pub streams: Vec<StreamInterest>,
}

impl InterestSnapshot {
    pub fn from_registry(registry: &Registry, requested: &[String]) -> Self {
        let streams = requested
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter_map(|stream| {
                registry
                    .interest(stream, MAX_INTEREST_KEYS_PER_STREAM)
                    .map(|interest| StreamInterest {
                        stream: stream.clone(),
                        all_keys: interest.all_keys,
                        keys: interest.keys,
                    })
            })
            .collect();
        Self { streams }
    }

    /// Restrict an untrusted peer response to requested streams, sort
    /// and deduplicate keys, and collapse oversized data to fail-open
    /// wildcard interest.
    fn sanitize(self, requested: &[String]) -> Self {
        use std::collections::{BTreeMap, BTreeSet};

        let requested: BTreeSet<&str> = requested.iter().map(String::as_str).collect();
        let mut merged: BTreeMap<String, StreamInterest> = BTreeMap::new();
        for mut interest in self.streams {
            if !requested.contains(interest.stream.as_str()) {
                continue;
            }
            interest.keys.sort();
            interest.keys.dedup();
            if interest.keys.len() > MAX_INTEREST_KEYS_PER_STREAM {
                interest.all_keys = true;
                interest.keys.clear();
            }
            let entry = merged
                .entry(interest.stream.clone())
                .or_insert_with(|| StreamInterest {
                    stream: interest.stream.clone(),
                    all_keys: false,
                    keys: Vec::new(),
                });
            if interest.all_keys {
                entry.all_keys = true;
                entry.keys.clear();
            } else if !entry.all_keys {
                entry.keys.extend(interest.keys);
                entry.keys.sort();
                entry.keys.dedup();
                if entry.keys.len() > MAX_INTEREST_KEYS_PER_STREAM {
                    entry.all_keys = true;
                    entry.keys.clear();
                }
            }
        }
        Self {
            streams: merged.into_values().collect(),
        }
    }

    pub fn wants(&self, event: &EventEnvelope) -> bool {
        self.streams.iter().any(|interest| {
            interest.stream == event.stream
                && (interest.all_keys
                    || event.key.as_ref().is_some_and(|key| {
                        interest
                            .keys
                            .binary_search_by(|candidate| candidate.as_str().cmp(key.as_str()))
                            .is_ok()
                    }))
        })
    }
}

/// Fire-and-forget relay of a producer batch to every discovered peer.
/// Best-effort and never awaited by the request path: backpressure or a
/// degraded peer must never reach the producer. With no peers it is a
/// no-op ⇒ byte-identical to a single instance.
pub fn spawn_relay(state: &AppState, batch: RelayBatch) {
    let peers = state.peers();
    if peers.is_empty() {
        return; // no peers ⇒ inert, byte-identical to single instance
    }
    state
        .metrics()
        .relay_events_relayed
        .inc_by(batch.events.len() as u64);

    if let Some(tx) = state.relay_tx() {
        // Coalescing on: enqueue; the flush task batches + POSTs.
        if tx.try_send(batch).is_err() {
            // Queue full (or closed) ⇒ drop this batch + meter. Never
            // inward backpressure (best-effort, at-most-once).
            state.metrics().relay_queue_dropped.inc();
        }
        return;
    }

    // Coalescing off: direct per-push task. Filtering and network I/O
    // remain detached from the producer path.
    let state = state.clone();
    tokio::spawn(async move {
        send_events_to_peers(&state, peers.as_ref(), &batch.events, usize::MAX).await;
    });
}

/// Refresh aggregate interest for the current peer set. Invalid,
/// unavailable, or non-success responses remove the cached entry so
/// relay falls back to unconditional broadcast. This is also the
/// rolling-upgrade path for peers that do not yet expose the endpoint.
pub async fn refresh_peer_interests(state: &AppState) {
    let Some(manifest) = state.manifest() else {
        state.clear_peer_interests();
        return;
    };
    let streams: Vec<String> = manifest
        .streams
        .iter()
        .filter(|stream| stream.selective_relay)
        .map(|stream| stream.name.clone())
        .collect();
    if streams.is_empty() {
        state.clear_peer_interests();
        return;
    }

    let peers = state.peers();
    if peers.is_empty() {
        state.clear_peer_interests();
        return;
    }
    let request = InterestRequest {
        streams: streams.clone(),
    };
    let mut body = Vec::new();
    if ciborium::into_writer(&request, &mut body).is_err() {
        state.clear_peer_interests();
        return;
    }

    let client = state.relay_client().clone();
    let token = state.config().push_auth_token.clone();
    let fetches = peers.iter().cloned().map(|peer| {
        let client = client.clone();
        let token = token.clone();
        let body = body.clone();
        let streams = streams.clone();
        async move {
            let response = client
                .post(format!("{}/internal/interest", peer.base()))
                .bearer_auth(token)
                .header(header::CONTENT_TYPE, "application/cbor")
                .body(body)
                .send()
                .await;
            let snapshot = match response {
                Ok(response) if response.status().is_success() => match response.bytes().await {
                    Ok(bytes) => ciborium::from_reader::<InterestSnapshot, _>(bytes.as_ref())
                        .ok()
                        .map(|snapshot| snapshot.sanitize(&streams)),
                    Err(_) => None,
                },
                _ => None,
            };
            (peer, snapshot)
        }
    });
    for (peer, snapshot) in futures_util::future::join_all(fetches).await {
        state
            .metrics()
            .relay_interest_refresh
            .get_or_create(&ReloadResultLabel {
                result: if snapshot.is_some() {
                    ReloadResult::Ok
                } else {
                    ReloadResult::Error
                },
            })
            .inc();
        state.set_peer_interest(&peer, snapshot);
    }
}

/// Spawn the relay flush task. No-op if coalescing is off (rx
/// absent). The loop body is panic-free by construction (every
/// network/encode error is metered, no `unwrap`s on the hot path),
/// so no in-process supervisor is needed. If the task does ever die,
/// `Sender::try_send` will return `Closed` for every subsequent
/// producer push — `relay_queue_dropped_total` climbs and
/// `relay_sent_total` flatlines, which is the operator-visible signal.
pub fn spawn_relay_flusher(state: AppState) {
    let Some(mut rx) = state.take_relay_rx() else {
        return;
    };
    tokio::spawn(async move {
        relay_flush_loop(&state, &mut rx).await;
    });
}

/// Drain the relay queue, batching by window-or-size, until the
/// channel closes (all senders dropped ⇒ shutdown).
async fn relay_flush_loop(state: &AppState, rx: &mut mpsc::Receiver<RelayBatch>) {
    let window = std::time::Duration::from_millis(state.config().relay_coalesce_ms.max(1));
    let max_events = state.config().relay_coalesce_max_events.max(1);
    loop {
        let Some(first) = rx.recv().await else {
            return; // all senders dropped → exit cleanly
        };
        let mut pending: Vec<EventEnvelope> = first.events;
        let deadline = tokio::time::Instant::now() + window;
        while pending.len() < max_events {
            tokio::select! {
                _ = tokio::time::sleep_until(deadline) => break,
                maybe = rx.recv() => match maybe {
                    Some(b) => pending.extend(b.events),
                    None => break, // closed: flush what we have, then exit
                },
            }
        }
        flush_to_peers(state, pending, max_events).await;
        state.metrics().relay_flushes.inc();
        state.metrics().relay_queue_depth.set(rx.len() as i64);
        if rx.is_closed() && rx.is_empty() {
            return;
        }
    }
}

/// POST `events` to every peer, chunked at `max_events` per request
/// (the blast-radius bound), all peers concurrently.
async fn flush_to_peers(state: &AppState, events: Vec<EventEnvelope>, max_events: usize) {
    let peers = state.peers();
    if peers.is_empty() || events.is_empty() {
        return;
    }
    send_events_to_peers(state, peers.as_ref(), &events, max_events).await;
}

/// Filter and POST one event set independently for every peer. A peer
/// with no valid interest snapshot receives the full set (fail open).
async fn send_events_to_peers(
    state: &AppState,
    peers: &[PeerUrl],
    events: &[EventEnvelope],
    max_events: usize,
) {
    let client = state.relay_client().clone();
    let token = state.config().push_auth_token.clone();
    let manifest = state.manifest();
    let mut sends = Vec::new();
    for peer in peers {
        let interest = state.peer_interest(peer);
        let filtered: Vec<EventEnvelope> = events
            .iter()
            .filter_map(|event| {
                let selective = manifest
                    .as_ref()
                    .and_then(|manifest| manifest.stream(&event.stream))
                    .is_some_and(|stream| stream.selective_relay);
                if selective
                    && interest
                        .as_ref()
                        .is_some_and(|snapshot| !snapshot.wants(event))
                {
                    state.metrics().relay_events_skipped.inc();
                    None
                } else {
                    Some(event.clone())
                }
            })
            .collect();

        for chunk in filtered.chunks(max_events.max(1)) {
            let mut body = Vec::new();
            if ciborium::into_writer(
                &RelayBatch {
                    events: chunk.to_vec(),
                },
                &mut body,
            )
            .is_err()
            {
                state
                    .metrics()
                    .relay_failed
                    .get_or_create(&RelayFailureLabel {
                        reason: RelayFailure::Other,
                    })
                    .inc();
                continue;
            }
            let url = format!("{}/internal/relay", peer.base());
            let client = client.clone();
            let token = token.clone();
            sends.push(async move {
                client
                    .post(&url)
                    .bearer_auth(&token)
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(body)
                    .send()
                    .await
            });
        }
    }

    for result in futures_util::future::join_all(sends).await {
        match result {
            Ok(resp) if resp.status().is_success() => {
                state.metrics().relay_sent.inc();
            }
            Ok(_) => {
                state
                    .metrics()
                    .relay_failed
                    .get_or_create(&RelayFailureLabel {
                        reason: RelayFailure::Status,
                    })
                    .inc();
            }
            Err(e) => {
                let reason = if e.is_timeout() {
                    RelayFailure::Timeout
                } else if e.is_connect() {
                    RelayFailure::Connect
                } else {
                    RelayFailure::Other
                };
                state
                    .metrics()
                    .relay_failed
                    .get_or_create(&RelayFailureLabel { reason })
                    .inc();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::peers::{PeerUrl, Scheme};

    fn coalescing_config() -> Config {
        let mut c = Config::new(
            "t".into(),
            crate::config::HandshakeKeyConfig {
                hs256_secret: Some("k".into()),
                ed25519_public_pem: None,
            },
            "p".into(),
        );
        c.queue_depth = 8;
        c.inbound_rate_per_sec = 0;
        c.inbound_burst = 0;
        c.relay_coalesce_ms = 10;
        c.relay_queue_depth = 4;
        c
    }

    fn a_peer() -> PeerUrl {
        PeerUrl {
            scheme: Scheme::Http,
            host: "127.0.0.1".into(),
            port: 9,
        }
    }

    fn one_event_batch() -> RelayBatch {
        RelayBatch {
            events: vec![EventEnvelope {
                stream: "s".into(),
                key: None,
                payload: serde_json::json!({}),
            }],
        }
    }

    #[tokio::test]
    async fn enqueue_when_coalescing_on_and_peers_present() {
        let state = AppState::new(coalescing_config());
        let mut rx = state.take_relay_rx().expect("rx");
        state.set_peers(vec![a_peer()]);
        spawn_relay(&state, one_event_batch());
        let got = rx.try_recv().expect("a batch was enqueued");
        assert_eq!(got.events.len(), 1);
    }

    #[tokio::test]
    async fn no_peers_short_circuits_even_with_coalescing_on() {
        let state = AppState::new(coalescing_config());
        let mut rx = state.take_relay_rx().expect("rx");
        // No peers ⇒ peers().is_empty() ⇒ nothing enqueued.
        spawn_relay(&state, one_event_batch());
        assert!(rx.try_recv().is_err(), "no peers ⇒ inert, nothing enqueued");
    }

    #[tokio::test]
    async fn full_queue_drops_and_meters() {
        let state = AppState::new(coalescing_config()); // relay_queue_depth: 4
        let _rx = state.take_relay_rx().expect("rx"); // never drained
        state.set_peers(vec![a_peer()]);
        for _ in 0..10 {
            spawn_relay(&state, one_event_batch());
        }
        let body = state.metrics().encode();
        assert!(
            body.contains("wss_mux_relay_queue_dropped_total")
                && !body.contains("wss_mux_relay_queue_dropped_total 0"),
            "expected non-zero queue-full drops metered, got:\n{body}"
        );
    }

    #[tokio::test]
    async fn flush_loop_batches_until_size_cap_then_posts_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        let posts = Arc::new(AtomicUsize::new(0));
        let events = Arc::new(AtomicUsize::new(0));
        let posts2 = posts.clone();
        let events2 = events.clone();

        let app = axum::Router::new().route(
            "/internal/relay",
            axum::routing::post(move |body: axum::body::Bytes| {
                let posts2 = posts2.clone();
                let events2 = events2.clone();
                async move {
                    let b: RelayBatch = ciborium::from_reader(body.as_ref()).unwrap();
                    posts2.fetch_add(1, Ordering::SeqCst);
                    events2.fetch_add(b.events.len(), Ordering::SeqCst);
                    axum::http::StatusCode::NO_CONTENT
                }
            }),
        );
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let peer_addr = l.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(l, app).await.unwrap() });

        let mut c = coalescing_config();
        c.relay_coalesce_ms = 30;
        c.relay_coalesce_max_events = 3; // size cap forces an early flush
        c.relay_queue_depth = 64; // headroom for the 6-event burst
        let state = AppState::new(c);
        state.set_peers(vec![PeerUrl {
            scheme: Scheme::Http,
            host: peer_addr.ip().to_string(),
            port: peer_addr.port(),
        }]);
        spawn_relay_flusher(state.clone());

        for _ in 0..6 {
            spawn_relay(&state, one_event_batch());
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        assert_eq!(events.load(Ordering::SeqCst), 6, "all events delivered");
        let p = posts.load(Ordering::SeqCst);
        assert!((1..=3).contains(&p), "coalesced into few POSTs, got {p}");
        assert!(state
            .metrics()
            .encode()
            .contains("wss_mux_relay_flushes_total"));
    }

    #[test]
    fn relay_batch_cbor_roundtrips() {
        let batch = RelayBatch {
            events: vec![
                EventEnvelope {
                    stream: "chat_messages".into(),
                    key: Some("room-42".into()),
                    payload: serde_json::json!({"text": "hi"}),
                },
                EventEnvelope {
                    stream: "presence".into(),
                    key: None,
                    payload: serde_json::json!({"who": "alice"}),
                },
            ],
        };
        let mut buf = Vec::new();
        ciborium::into_writer(&batch, &mut buf).expect("cbor encode");
        let back: RelayBatch = ciborium::from_reader(&buf[..]).expect("cbor decode");
        assert_eq!(back, batch);
    }

    #[test]
    fn interest_snapshot_matches_wildcard_and_exact_keys() {
        let snapshot = InterestSnapshot {
            streams: vec![
                StreamInterest {
                    stream: "all".into(),
                    all_keys: true,
                    keys: vec![],
                },
                StreamInterest {
                    stream: "keyed".into(),
                    all_keys: false,
                    keys: vec!["a".into(), "b".into()],
                },
            ],
        };
        let event = |stream: &str, key: Option<&str>| EventEnvelope {
            stream: stream.into(),
            key: key.map(str::to_string),
            payload: serde_json::json!({}),
        };
        assert!(snapshot.wants(&event("all", None)));
        assert!(snapshot.wants(&event("all", Some("anything"))));
        assert!(snapshot.wants(&event("keyed", Some("b"))));
        assert!(!snapshot.wants(&event("keyed", Some("c"))));
        assert!(!snapshot.wants(&event("keyed", None)));
        assert!(!snapshot.wants(&event("missing", Some("a"))));
    }

    #[test]
    fn sanitizing_oversized_peer_interest_fails_open() {
        let snapshot = InterestSnapshot {
            streams: vec![StreamInterest {
                stream: "s".into(),
                all_keys: false,
                keys: (0..=MAX_INTEREST_KEYS_PER_STREAM)
                    .map(|i| format!("key-{i}"))
                    .collect(),
            }],
        }
        .sanitize(&["s".into()]);
        assert_eq!(snapshot.streams.len(), 1);
        assert!(snapshot.streams[0].all_keys);
        assert!(snapshot.streams[0].keys.is_empty());
    }

    #[tokio::test]
    async fn failed_interest_refresh_clears_snapshot_to_fail_open() {
        let mut config = coalescing_config();
        config.relay_coalesce_ms = 0;
        let state = AppState::new(config);
        state.set_manifest(
            crate::manifest::Manifest::from_str(
                "version: 1\nstreams:\n  - stream: s\n    subscribe: [role:x]\n    selective_relay: true\n",
                std::path::Path::new("test.yaml"),
            )
            .expect("manifest"),
        );
        let peer = a_peer();
        state.set_peers(vec![peer.clone()]);
        state.set_peer_interest(&peer, Some(InterestSnapshot::default()));
        assert!(state.peer_interest(&peer).is_some());

        refresh_peer_interests(&state).await;
        assert!(state.peer_interest(&peer).is_none());
        assert!(state
            .metrics()
            .encode()
            .contains("wss_mux_relay_interest_refresh_total{result=\"error\"} 1"));
    }
}
