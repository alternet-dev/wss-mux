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

/// Fan an event out to every matching subscription. v0.4 model (c):
/// each subscription owns its own channel (sized at subscribe time from
/// the stream's effective `queue_depth`). The dispatcher just sends to
/// that channel — no per-connection shared queue, no in-flight
/// accounting. A full per-sub channel is that subscription overflowing
/// on its own; the connection and its other subscriptions are
/// untouched.
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
        let Some((sender, cap)) = state.sub_sender(conn_id, &sub_id) else {
            // No live channel for this subscription (torn down, or not
            // yet registered with the writer) — treat as closed.
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

        // send-queue-depth metric: meaningful only for a bounded sub
        // (`cap == 0` ⇒ unbounded/unlimited, no depth).
        if cap != 0 {
            if let Some(remaining) = sender.capacity() {
                state
                    .metrics()
                    .send_queue_depth
                    .observe(cap.saturating_sub(remaining) as f64);
            }
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
                // This subscription's own channel is full → per-sub
                // overflow. The keep-open error rides the connection
                // *control* channel (the sub's channel is the thing
                // that's full); the subscription alone is dropped, the
                // connection and its other subscriptions keep running.
                if let Some(ctl) = state.control_sender(conn_id) {
                    let _ =
                        ctl.try_send(ProtocolError::Overflow { id: sub_id.clone() }.to_outbound());
                }
                state.remove_sub_sender(conn_id, &sub_id);
                state
                    .registry()
                    .unsubscribe(&envelope.stream, conn_id, &sub_id);
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
                // Receiver gone (writer evicted it / connection tearing
                // down). Drop the stale entry; count as closed.
                state.remove_sub_sender(conn_id, &sub_id);
                state
                    .registry()
                    .unsubscribe(&envelope.stream, conn_id, &sub_id);
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
    use crate::connection::outbound_channel;
    use serde_json::json;

    fn test_config(queue_depth: usize) -> Config {
        Config {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            push_auth_token: "t".into(),
            handshake_keys: crate::config::HandshakeKeyConfig {
                hs256_secret: Some("k".into()),
                ed25519_public_pem: None,
            },
            oidc: None,
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

    fn env(stream: &str) -> EventEnvelope {
        EventEnvelope {
            stream: stream.into(),
            key: None,
            payload: json!({}),
        }
    }

    /// Register a subscription end-to-end the way the reader does: a
    /// per-sub channel of `cap` plus a binding in the registry.
    fn subscribe(
        state: &AppState,
        conn: u64,
        sub: &str,
        stream: &str,
        cap: usize,
    ) -> OutboundRxHandle {
        let (tx, rx) = outbound_channel(cap);
        state.register_sub_sender(conn, sub, tx, cap);
        state.registry().subscribe(stream, conn, sub.into(), None);
        OutboundRxHandle(rx)
    }

    // Keeps the per-sub receiver alive in tests (dropping it would make
    // try_send return Closed). `recv` drains synchronously.
    struct OutboundRxHandle(crate::connection::OutboundRx);
    impl OutboundRxHandle {
        fn try_drain(&mut self) -> Vec<Outbound> {
            let mut out = Vec::new();
            while let Ok(v) = match &mut self.0 {
                crate::connection::OutboundRx::Bounded(r) => r.try_recv(),
                crate::connection::OutboundRx::Unbounded(r) => r.try_recv(),
            } {
                out.push(v);
            }
            out
        }
    }

    #[test]
    fn no_subscribers_increments_dropped_metric() {
        let state = AppState::new(test_config(8));
        let stats = dispatch(&state, env("chat_messages"));
        assert_eq!(stats.delivered, 0);
        let body = state.metrics().encode();
        assert!(
            body.contains("wss_mux_events_dropped_total{reason=\"no_subscribers\"} 1"),
            "expected no_subscribers drop in metrics body, got:\n{body}"
        );
    }

    #[test]
    fn happy_path_delivers_to_the_subscription_channel() {
        let state = AppState::new(test_config(8));
        let mut rx = subscribe(&state, 1, "s1", "chat_messages", 8);

        let stats = dispatch(
            &state,
            EventEnvelope {
                stream: "chat_messages".into(),
                key: None,
                payload: json!({"hello": "world"}),
            },
        );

        assert_eq!(stats.delivered, 1);
        let drained = rx.try_drain();
        assert_eq!(drained.len(), 1);
        let Outbound::Frame(ServerFrame::Event { id, .. }) = &drained[0] else {
            panic!("expected event frame");
        };
        assert_eq!(id, "s1");
    }

    #[test]
    fn closed_subscription_channel_counts_as_dropped_closed() {
        let state = AppState::new(test_config(8));
        let rx = subscribe(&state, 7, "s1", "chat_messages", 8);
        drop(rx); // writer side gone

        let stats = dispatch(&state, env("chat_messages"));
        assert_eq!(stats.dropped_closed, 1);
        // Stale sender pruned; binding cleaned.
        assert!(state.sub_sender(7, "s1").is_none());
        assert!(state.registry().matches("chat_messages", None).is_empty());
    }

    #[test]
    fn full_sub_channel_overflows_alone_via_control_and_drops_only_that_sub() {
        let state = AppState::new(test_config(2));
        // Connection control channel (where the keep-open overflow
        // error must be delivered).
        let (ctl_tx, ctl_rx) = outbound_channel(0);
        state.register_control(1, ctl_tx);
        let mut ctl = OutboundRxHandle(ctl_rx);
        // s1 cap 2, never drained → fills then overflows. s2 healthy.
        let mut rx1 = subscribe(&state, 1, "s1", "chat_messages", 2);
        let mut rx2 = subscribe(&state, 1, "s2", "other", 8);

        for _ in 0..2 {
            dispatch(&state, env("chat_messages"));
        }
        // 3rd: s1 channel is full → per-sub overflow.
        let stats = dispatch(&state, env("chat_messages"));
        assert_eq!(stats.dropped_full, 1);
        assert_eq!(stats.delivered, 0);

        // Only s1 is gone; the connection (control) and s2 survive.
        assert!(state.control_sender(1).is_some());
        assert!(state.sub_sender(1, "s1").is_none());
        assert!(state.registry().matches("chat_messages", None).is_empty());
        dispatch(&state, env("other"));
        assert_eq!(rx2.try_drain().len(), 1, "s2 keeps delivering");

        // Client got a keep-open overflow error for s1 on the control
        // channel.
        let saw = ctl.try_drain().into_iter().any(|o| {
            matches!(o, Outbound::Frame(ServerFrame::Error { code, id, .. })
                if code == "overflow" && id.as_deref() == Some("s1"))
        });
        assert!(saw, "expected overflow error for s1 on the control channel");
        let _ = rx1.try_drain();
    }

    #[test]
    fn unbounded_sub_never_overflows() {
        let state = AppState::new(test_config(0));
        // cap 0 ⇒ unbounded per-sub channel; nothing drains it.
        let mut rx = subscribe(&state, 1, "s1", "chat_messages", 0);
        for i in 0..1000 {
            let s = dispatch(
                &state,
                EventEnvelope {
                    stream: "chat_messages".into(),
                    key: None,
                    payload: json!({ "i": i }),
                },
            );
            assert_eq!(s.delivered, 1);
            assert_eq!(s.dropped_full, 0);
        }
        assert!(state.registry().matches("chat_messages", None).len() == 1);
        assert_eq!(rx.try_drain().len(), 1000);
    }
}
