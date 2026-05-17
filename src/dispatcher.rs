use crate::connection::{Outbound, SendError};
use crate::envelope::{EventEnvelope, ServerFrame};
use crate::error::ProtocolError;
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

    // Effective per-subscription in-flight cap. `None` = unlimited:
    // the stream's manifest `queue_depth` if set (0 ⇒ unlimited), else
    // the global `WSS_MUX_QUEUE_DEPTH` (0 ⇒ unlimited). An unlimited
    // subscription skips the reservation gate entirely.
    let cap: Option<usize> = match state
        .manifest()
        .and_then(|m| m.stream(&envelope.stream).and_then(|s| s.queue_depth))
    {
        Some(0) => None,
        Some(n) => Some(n),
        None => match state.config().queue_depth {
            0 => None,
            n => Some(n),
        },
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

        if let Some(cap) = cap {
            if !state.sub_queues().try_reserve(conn_id, &sub_id, cap) {
                // Subscription is at its in-flight cap: per-subscription
                // overflow. Drop only this subscription — keep-open
                // error frame, unsubscribe it, forget its accounting.
                // The connection and its other subscriptions are
                // unaffected.
                let _ =
                    sender.try_send(ProtocolError::Overflow { id: sub_id.clone() }.to_outbound());
                state
                    .registry()
                    .unsubscribe(&envelope.stream, conn_id, &sub_id);
                state.sub_queues().drop_sub(conn_id, &sub_id);
                stats.dropped_full += 1;
                state
                    .metrics()
                    .events_dropped
                    .get_or_create(&DropReasonLabel {
                        reason: DropReason::Overflow,
                    })
                    .inc();
                continue;
            }
        }
        // cap == None ⇒ unlimited: no reservation, never per-sub
        // overflow for depth (bounded only by the shared channel).

        // Depth metric only applies to a bounded channel; an unbounded
        // (unlimited) connection has no meaningful queue depth.
        if let Some(remaining) = sender.capacity() {
            let queue_used = state.config().queue_depth.saturating_sub(remaining);
            state.metrics().send_queue_depth.observe(queue_used as f64);
        }

        let frame = ServerFrame::Event {
            id: sub_id.clone(),
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
            Err(SendError::Full) => {
                // Shared per-connection channel is saturated. The slot
                // reserved above never reached the writer (which would
                // release it), so release it here. The frame is dropped
                // (at-most-once); the connection and subscription both
                // survive — a full shared channel is no longer a
                // connection-level overflow close.
                state.sub_queues().release(conn_id, &sub_id);
                stats.dropped_full += 1;
                state
                    .metrics()
                    .events_dropped
                    .get_or_create(&DropReasonLabel {
                        reason: DropReason::Overflow,
                    })
                    .inc();
            }
            Err(SendError::Closed) => {
                // Writer is gone; release the slot we just reserved.
                state.sub_queues().release(conn_id, &sub_id);
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
    use crate::connection::OutboundTx;
    use crate::manifest::Manifest;
    use serde_json::json;
    use std::path::Path;
    use tokio::sync::mpsc;

    fn test_config(queue_depth: usize) -> Config {
        Config {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            push_auth_token: "t".into(),
            handshake_keys: crate::config::HandshakeKeyConfig {
                hs256_secret: Some("k".into()),
                ed25519_public_pem: None,
            },
            manifest_path: "p".into(),
            queue_depth,
            envelope_stream_path: "stream".into(),
            envelope_key_path: "key".into(),
            envelope_payload_path: "payload".into(),
            inbound_rate_per_sec: 0,
            inbound_burst: 0,
            peers: crate::config::PeerConfig::default(),
        }
    }

    #[test]
    fn closed_channel_counts_as_dropped_closed() {
        let state = AppState::new(test_config(1));
        let (data_tx, data_rx) = mpsc::channel::<Outbound>(1);
        state.register_connection(7, OutboundTx::Bounded(data_tx));
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
        // Connection survives — closed-channel is not a connection close.
        assert!(state.sender(7).is_some());
    }

    #[test]
    fn happy_path_delivers_to_subscribed_connection() {
        let state = AppState::new(test_config(8));
        let (data_tx, mut data_rx) = mpsc::channel::<Outbound>(8);
        state.register_connection(1, OutboundTx::Bounded(data_tx));
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
    fn dispatch_reserves_a_subscription_slot() {
        let state = AppState::new(test_config(8));
        let (data_tx, _data_rx) = mpsc::channel::<Outbound>(8);
        state.register_connection(1, OutboundTx::Bounded(data_tx));
        state
            .registry()
            .subscribe("chat_messages", 1, "s1".into(), None);

        let stats = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({"n": 1}),
            },
        );

        assert_eq!(stats.delivered, 1);
        // The dispatcher reserved one per-subscription in-flight slot;
        // with no writer in this unit test it stays reserved.
        assert_eq!(state.sub_queues().in_flight(1, "s1"), 1);
    }

    #[test]
    fn subscription_at_cap_overflows_alone_and_connection_survives() {
        // cap (no manifest) == global queue_depth == 2.
        let state = AppState::new(test_config(2));
        let (data_tx, mut data_rx) = mpsc::channel::<Outbound>(8);
        state.register_connection(1, OutboundTx::Bounded(data_tx));
        state
            .registry()
            .subscribe("chat_messages", 1, "s1".into(), None);

        // Fill the subscription's in-flight to its cap (no writer here,
        // so reservations are never released).
        for _ in 0..2 {
            dispatch(
                &state,
                EventEnvelope {
                    stream: "chat_messages".into(),
                    key: None,
                    payload: json!({}),
                },
            );
        }
        assert_eq!(state.sub_queues().in_flight(1, "s1"), 2);

        // One more event: the subscription is at cap → per-subscription
        // overflow. The connection and its other subscriptions survive.
        let stats = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({"overflowing": true}),
            },
        );

        assert_eq!(stats.delivered, 0);
        assert_eq!(stats.dropped_full, 1);
        // Connection is NOT closed (contrast pre-v0.4 4429 behavior).
        assert!(state.sender(1).is_some(), "connection must stay open");
        // Only this subscription is gone.
        assert!(state.registry().matches("chat_messages", None).is_empty());
        assert_eq!(
            state.sub_queues().in_flight(1, "s1"),
            0,
            "drop_sub clears the overflowed subscription's accounting"
        );

        // The client received a keep-open overflow error naming s1.
        let mut saw_overflow = false;
        while let Ok(out) = data_rx.try_recv() {
            if let Outbound::Frame(ServerFrame::Error { code, id, .. }) = out {
                assert_eq!(code, "overflow");
                assert_eq!(id.as_deref(), Some("s1"));
                saw_overflow = true;
            }
        }
        assert!(saw_overflow, "expected an overflow error frame for s1");
    }

    #[test]
    fn shared_channel_full_drops_frame_but_keeps_connection_and_subscription() {
        // cap is high (64) so the per-subscription gate is not the
        // limiter here; the small shared channel is.
        let state = AppState::new(test_config(64));
        let (data_tx, _data_rx) = mpsc::channel::<Outbound>(1);
        state.register_connection(1, OutboundTx::Bounded(data_tx.clone()));
        state
            .registry()
            .subscribe("chat_messages", 1, "s1".into(), None);

        // Saturate the shared per-connection channel so the
        // dispatcher's try_send returns Full.
        data_tx
            .try_send(Outbound::Frame(ServerFrame::Event {
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

        assert_eq!(stats.delivered, 0);
        assert_eq!(stats.dropped_full, 1);
        // The reservation made before try_send is released — not leaked.
        assert_eq!(
            state.sub_queues().in_flight(1, "s1"),
            0,
            "reservation must be released when the frame is not enqueued"
        );
        // A full shared channel no longer closes the connection
        // (pre-v0.4 this was a 4429 close).
        assert!(
            state.sender(1).is_some(),
            "connection survives a full shared channel"
        );
        // The subscription was never over its own cap, so it stays.
        assert_eq!(
            state.registry().matches("chat_messages", None),
            vec![(1u64, "s1".to_string())]
        );
    }

    #[test]
    fn per_stream_queue_depth_overrides_global_cap() {
        // Global queue_depth is 64; the stream caps itself at 1 via the
        // manifest. The per-stream value must be the effective cap.
        let state = AppState::new(test_config(64));
        state.set_manifest(
            Manifest::from_str(
                "version: 1\n\
                 streams:\n  \
                 - stream: chat_messages\n    \
                 audience: [role:x]\n    \
                 queue_depth: 1\n",
                Path::new("test.yaml"),
            )
            .expect("valid manifest"),
        );
        let (data_tx, _data_rx) = mpsc::channel::<Outbound>(8);
        state.register_connection(1, OutboundTx::Bounded(data_tx));
        state
            .registry()
            .subscribe("chat_messages", 1, "s1".into(), None);

        // First event fits the per-stream cap of 1.
        let s = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({}),
            },
        );
        assert_eq!(s.delivered, 1);
        assert_eq!(state.sub_queues().in_flight(1, "s1"), 1);

        // Second event: already at the manifest cap of 1 (NOT the
        // global 64) → per-subscription overflow.
        let s = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({}),
            },
        );
        assert_eq!(s.delivered, 0);
        assert_eq!(s.dropped_full, 1);
        assert!(
            state.registry().matches("chat_messages", None).is_empty(),
            "manifest queue_depth:1 must cap this stream at 1 in-flight, \
             not the global 64"
        );
    }

    #[test]
    fn queue_depth_zero_means_unlimited_no_per_sub_cap() {
        // Stream sets queue_depth: 0 → explicit unlimited. The
        // dispatcher must skip the per-subscription reservation
        // entirely: no overflow, no in-flight accounting, every event
        // delivered (bounded only by the shared channel, ample here).
        let state = AppState::new(test_config(4));
        state.set_manifest(
            Manifest::from_str(
                "version: 1\n\
                 streams:\n  \
                 - stream: chat_messages\n    \
                 audience: [role:x]\n    \
                 queue_depth: 0\n",
                Path::new("test.yaml"),
            )
            .expect("valid manifest"),
        );
        let (data_tx, _data_rx) = mpsc::channel::<Outbound>(64);
        state.register_connection(1, OutboundTx::Bounded(data_tx));
        state
            .registry()
            .subscribe("chat_messages", 1, "s1".into(), None);

        for i in 0..20 {
            let s = dispatch(
                &state,
                EventEnvelope {
                    stream: "chat_messages".into(),
                    key: None,
                    payload: json!({ "i": i }),
                },
            );
            assert_eq!(s.delivered, 1, "event {i} must be delivered");
            assert_eq!(s.dropped_full, 0, "unlimited stream must not overflow");
        }
        // No reservation is ever taken for an unlimited subscription.
        assert_eq!(state.sub_queues().in_flight(1, "s1"), 0);
        // Subscription is still alive (never overflow-dropped).
        assert_eq!(
            state.registry().matches("chat_messages", None),
            vec![(1u64, "s1".to_string())]
        );
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
