//! Outbound peer-relay: the relay wire batch, the per-push relay
//! decision, and (v0.5) the coalescing flush worker + supervisor.

use axum::http::header;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::peers::{PeerUrl, Scheme};

    fn coalescing_config() -> Config {
        Config {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            push_auth_token: "t".into(),
            handshake_keys: crate::config::HandshakeKeyConfig {
                hs256_secret: Some("k".into()),
                ed25519_public_pem: None,
            },
            oidc: None,
            manifest_path: "p".into(),
            queue_depth: 8,
            envelope_stream_path: "stream".into(),
            envelope_key_path: "key".into(),
            envelope_payload_path: "payload".into(),
            inbound_rate_per_sec: 0,
            inbound_burst: 0,
            relay_coalesce_ms: 10,
            relay_coalesce_max_events: 1024,
            relay_queue_depth: 4,
            peers: crate::config::PeerConfig::default(),
        }
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
