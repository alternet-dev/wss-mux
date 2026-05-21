//! Outbound peer-relay: the relay wire batch, the per-push relay
//! decision, and (v0.5) the coalescing flush worker.

use axum::http::header;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::envelope::EventEnvelope;
use crate::server::metrics::{RelayFailure, RelayFailureLabel};
use crate::server::AppState;

/// Internal relay body: a batch of already-canonical envelopes,
/// CBOR-encoded. Independent of the producer-facing `WSS_MUX_ENVELOPE_*`
/// config — peers exchange canonical envelopes, so receiving a relay
/// needs no envelope-path coupling. A single producer event relays as a
/// one-element batch so relay code stays uniform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelayBatch {
    pub events: Vec<EventEnvelope>,
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

    // Coalescing off: pre-v0.5 direct per-push spawn (unchanged).
    let mut body = Vec::new();
    if ciborium::into_writer(&batch, &mut body).is_err() {
        state
            .metrics()
            .relay_failed
            .get_or_create(&RelayFailureLabel {
                reason: RelayFailure::Other,
            })
            .inc();
        return;
    }

    let client = state.relay_client().clone();
    let token = state.config().push_auth_token.clone();
    let state = state.clone();
    tokio::spawn(async move {
        for peer in peers.iter() {
            let url = format!("{}/internal/v1/relay", peer.base());
            let result = client
                .post(&url)
                .bearer_auth(&token)
                .header(header::CONTENT_TYPE, "application/cbor")
                .body(body.clone())
                .send()
                .await;
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
    });
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
    let client = state.relay_client().clone();
    let token = state.config().push_auth_token.clone();
    for chunk in events.chunks(max_events) {
        let batch = RelayBatch {
            events: chunk.to_vec(),
        };
        let mut body = Vec::new();
        if ciborium::into_writer(&batch, &mut body).is_err() {
            state
                .metrics()
                .relay_failed
                .get_or_create(&RelayFailureLabel {
                    reason: RelayFailure::Other,
                })
                .inc();
            continue;
        }
        let sends = peers.iter().map(|p| {
            let url = format!("{}/internal/v1/relay", p.base());
            let client = client.clone();
            let token = token.clone();
            let body = body.clone();
            async move {
                client
                    .post(&url)
                    .bearer_auth(&token)
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(body)
                    .send()
                    .await
            }
        });
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
            "/internal/v1/relay",
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
}
