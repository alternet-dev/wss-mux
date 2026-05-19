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
        return;
    }

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
