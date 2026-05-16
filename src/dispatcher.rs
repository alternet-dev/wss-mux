use crate::connection::Outbound;
use crate::envelope::{EventEnvelope, ServerFrame};
use crate::server::metrics::{DropReason, DropReasonLabel, StreamLabel};
use crate::server::AppState;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DispatchStats {
    pub delivered: usize,
    pub dropped_full: usize,
    pub dropped_closed: usize,
}

pub fn dispatch(state: &AppState, envelope: EventEnvelope) -> DispatchStats {
    let mut stats = DispatchStats::default();
    let matches = state
        .registry()
        .matches(&envelope.stream, envelope.key.as_deref());

    if matches.is_empty() {
        state
            .metrics()
            .events_dropped
            .get_or_create(&DropReasonLabel {
                reason: DropReason::NoSubscribers,
            })
            .inc();
        return stats;
    }

    let stream_label = StreamLabel {
        stream: envelope.stream.clone(),
    };

    for (conn_id, sub_id) in matches {
        let Some(sender) = state.sender(conn_id) else {
            stats.dropped_closed += 1;
            state
                .metrics()
                .events_dropped
                .get_or_create(&DropReasonLabel {
                    reason: DropReason::ChannelClosed,
                })
                .inc();
            continue;
        };

        let queue_used = state.config().queue_depth.saturating_sub(sender.capacity());
        state.metrics().send_queue_depth.observe(queue_used as f64);

        let frame = ServerFrame::Event {
            id: sub_id,
            stream: envelope.stream.clone(),
            key: envelope.key.clone(),
            payload: envelope.payload.clone(),
        };
        match sender.try_send(Outbound::Frame(frame)) {
            Ok(()) => {
                stats.delivered += 1;
                state
                    .metrics()
                    .events_dispatched
                    .get_or_create(&stream_label)
                    .inc();
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                stats.dropped_full += 1;
                state
                    .metrics()
                    .events_dropped
                    .get_or_create(&DropReasonLabel {
                        reason: DropReason::Overflow,
                    })
                    .inc();
                state.trigger_overflow(conn_id);
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                stats.dropped_closed += 1;
                state
                    .metrics()
                    .events_dropped
                    .get_or_create(&DropReasonLabel {
                        reason: DropReason::ChannelClosed,
                    })
                    .inc();
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use serde_json::json;
    use tokio::sync::{mpsc, watch};

    fn test_config(queue_depth: usize) -> Config {
        Config {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            push_auth_token: "t".into(),
            handshake_signing_key: "k".into(),
            manifest_path: "p".into(),
            queue_depth,
            envelope_stream_path: "stream".into(),
            envelope_key_path: "key".into(),
            envelope_payload_path: "payload".into(),
        }
    }

    #[test]
    fn full_channel_triggers_overflow() {
        let state = AppState::new(test_config(1));
        let (data_tx, _data_rx_kept_alive) = mpsc::channel::<Outbound>(1);
        let (abort_tx, abort_rx) = watch::channel(false);
        let conn_id = 42;
        state.register_connection(conn_id, data_tx.clone(), abort_tx);
        state
            .registry()
            .subscribe("chat_messages", conn_id, "s1".into(), None);

        // Pre-fill the channel so the dispatcher's try_send returns Full.
        data_tx
            .try_send(Outbound::Frame(crate::envelope::ServerFrame::Event {
                id: "preload".into(),
                stream: "chat_messages".into(),
                key: None,
                payload: json!({}),
            }))
            .unwrap();

        let stats = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({"x": 1}),
            },
        );

        assert_eq!(stats.dropped_full, 1);
        assert_eq!(stats.delivered, 0);
        // Connection was removed from the senders map.
        assert!(state.sender(conn_id).is_none());
        // Abort signal flipped to true.
        assert!(*abort_rx.borrow());
    }

    #[test]
    fn closed_channel_counts_as_dropped_closed() {
        let state = AppState::new(test_config(1));
        let (data_tx, data_rx) = mpsc::channel::<Outbound>(1);
        let (abort_tx, abort_rx) = watch::channel(false);
        state.register_connection(7, data_tx, abort_tx);
        state
            .registry()
            .subscribe("chat_messages", 7, "s1".into(), None);
        drop(data_rx); // simulate the writer task ending

        let stats = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({}),
            },
        );

        assert_eq!(stats.dropped_closed, 1);
        // Closed-channel path is NOT overflow, so abort stays false.
        assert!(!*abort_rx.borrow());
    }

    #[test]
    fn happy_path_delivers_to_subscribed_connection() {
        let state = AppState::new(test_config(8));
        let (data_tx, mut data_rx) = mpsc::channel::<Outbound>(8);
        let (abort_tx, _abort_rx) = watch::channel(false);
        state.register_connection(1, data_tx, abort_tx);
        state
            .registry()
            .subscribe("chat_messages", 1, "s1".into(), None);

        let stats = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({"hello": "world"}),
            },
        );

        assert_eq!(stats.delivered, 1);
        let out = data_rx.try_recv().expect("event delivered");
        let Outbound::Frame(crate::envelope::ServerFrame::Event { id, .. }) = out else {
            panic!("expected event frame");
        };
        assert_eq!(id, "s1");
    }

    #[test]
    fn no_subscribers_increments_dropped_metric() {
        let state = AppState::new(test_config(8));

        let stats = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({}),
            },
        );

        assert_eq!(stats.delivered, 0);
        // Verify metric incremented by encoding and string-checking.
        let body = state.metrics().encode();
        assert!(
            body.contains("wss_mux_events_dropped_total{reason=\"no_subscribers\"} 1"),
            "expected no_subscribers drop in metrics body, got:\n{body}"
        );
    }
}
