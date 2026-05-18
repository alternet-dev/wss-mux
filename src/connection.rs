use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::sync::mpsc;
use tokio_stream::Stream;

use crate::envelope::ServerFrame;

pub type ConnId = u64;

/// Outbound is what the writer task drains from the per-connection queue.
///
/// `Frame` is a normal server-to-client frame (an event, or a keep-open
/// error frame). `Close` is the final message on the connection: the
/// writer emits the optional accompanying frame, then sends a WebSocket
/// close with the given code, then exits.
#[derive(Debug, Clone)]
pub enum Outbound {
    Frame(ServerFrame),
    Close {
        code: u16,
        reason: String,
        frame: Option<ServerFrame>,
    },
}

impl Outbound {
    pub fn normal_close() -> Self {
        Self::Close {
            code: 1000,
            reason: String::new(),
            frame: None,
        }
    }
}

/// Per-connection outbound sender, unified over a bounded queue and an
/// unbounded one. A `queue_depth` of `0` (the explicit "unlimited"
/// signal) builds the unbounded variant so a connection configured
/// unlimited never sheds frames for depth.
#[derive(Clone)]
pub enum OutboundTx {
    Bounded(mpsc::Sender<Outbound>),
    Unbounded(mpsc::UnboundedSender<Outbound>),
}

/// Receiver half of [`OutboundTx`], drained by the writer task.
pub enum OutboundRx {
    Bounded(mpsc::Receiver<Outbound>),
    Unbounded(mpsc::UnboundedReceiver<Outbound>),
}

/// Why a non-blocking send failed. `Full` can only arise on a bounded
/// channel; an unbounded channel only fails once the receiver is gone.
#[derive(Debug)]
pub enum SendError {
    Full,
    Closed,
}

/// Build the per-connection channel. `depth == 0` ⇒ unbounded (the
/// explicit "unlimited" signal); otherwise a bounded channel of that
/// capacity. (`mpsc::channel(0)` would panic, so the branch is
/// load-bearing, not just an optimization.)
pub fn outbound_channel(depth: usize) -> (OutboundTx, OutboundRx) {
    if depth == 0 {
        let (tx, rx) = mpsc::unbounded_channel();
        (OutboundTx::Unbounded(tx), OutboundRx::Unbounded(rx))
    } else {
        let (tx, rx) = mpsc::channel(depth);
        (OutboundTx::Bounded(tx), OutboundRx::Bounded(rx))
    }
}

impl OutboundTx {
    /// Non-blocking send. Never blocks the dispatcher; a bounded
    /// channel at capacity returns `SendError::Full`.
    pub fn try_send(&self, item: Outbound) -> Result<(), SendError> {
        match self {
            Self::Bounded(tx) => tx.try_send(item).map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => SendError::Full,
                mpsc::error::TrySendError::Closed(_) => SendError::Closed,
            }),
            Self::Unbounded(tx) => tx.send(item).map_err(|_| SendError::Closed),
        }
    }

    /// Remaining slots on a bounded channel; `None` when unbounded
    /// (no meaningful depth — the connection is configured unlimited).
    pub fn capacity(&self) -> Option<usize> {
        match self {
            Self::Bounded(tx) => Some(tx.capacity()),
            Self::Unbounded(_) => None,
        }
    }
}

impl OutboundRx {
    pub async fn recv(&mut self) -> Option<Outbound> {
        match self {
            Self::Bounded(rx) => rx.recv().await,
            Self::Unbounded(rx) => rx.recv().await,
        }
    }
}

/// Lets the writer merge a dynamic set of per-subscription receivers
/// in a `tokio_stream::StreamMap`. A receiver yields `None` (ending
/// the stream, so `StreamMap` evicts it) once all its senders are
/// dropped — which is exactly how unsubscribe / overflow / connection
/// teardown remove a subscription from the writer.
impl Stream for OutboundRx {
    type Item = Outbound;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Outbound>> {
        // Both receivers are `Unpin`, so `get_mut` is sound.
        match self.get_mut() {
            Self::Bounded(rx) => rx.poll_recv(cx),
            Self::Unbounded(rx) => rx.poll_recv(cx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> Outbound {
        Outbound::Frame(ServerFrame::Error {
            code: "x".into(),
            message: "x".into(),
            id: None,
        })
    }

    #[test]
    fn zero_depth_is_unbounded_and_never_full() {
        let (tx, _rx) = outbound_channel(0);
        assert!(matches!(tx, OutboundTx::Unbounded(_)));
        // Far more than any bounded default, with nothing draining it.
        for _ in 0..10_000 {
            tx.try_send(frame())
                .expect("an unbounded channel never returns Full");
        }
    }

    #[tokio::test]
    async fn outbound_rx_is_a_stream_that_ends_when_senders_drop() {
        use tokio_stream::StreamExt;
        let (tx, mut rx) = outbound_channel(4);
        tx.try_send(frame()).unwrap();
        assert!(matches!(rx.next().await, Some(Outbound::Frame(_))));
        drop(tx);
        assert!(
            rx.next().await.is_none(),
            "the stream must end once every sender is dropped"
        );
    }

    #[tokio::test]
    async fn streammap_merges_per_sub_receivers_and_evicts_closed() {
        use std::collections::HashSet;
        use tokio_stream::{StreamExt, StreamMap};

        // The writer relies on exactly this: a dynamic set of per-sub
        // receivers merged fairly, with a sub auto-evicted when its
        // senders drop (unsubscribe / overflow / teardown).
        let (tx_a, rx_a) = outbound_channel(4);
        let (tx_b, rx_b) = outbound_channel(0); // an "unlimited" sub
        let mut map: StreamMap<String, OutboundRx> = StreamMap::new();
        map.insert("a".into(), rx_a);
        map.insert("b".into(), rx_b);

        tx_a.try_send(frame()).unwrap();
        tx_b.try_send(frame()).unwrap();
        let mut seen = HashSet::new();
        for _ in 0..2 {
            let (k, _v) = map.next().await.expect("merged item");
            seen.insert(k);
        }
        assert_eq!(seen.len(), 2, "both per-sub streams are merged");

        drop(tx_a); // sub "a" goes away (unsub / overflow / teardown)
        for _ in 0..4 {
            tx_b.try_send(frame()).unwrap();
        }
        // "a" must never yield again; "b" keeps delivering.
        for _ in 0..4 {
            let (k, _) = map.next().await.expect("b still delivers");
            assert_eq!(k, "b", "the closed sub must not yield");
        }
        drop(tx_b);
        assert!(
            map.next().await.is_none(),
            "the map terminates once every sub stream is closed (writer exits)"
        );
    }

    #[test]
    fn nonzero_depth_is_bounded_and_fulls_at_capacity() {
        let (tx, _rx) = outbound_channel(2);
        assert!(matches!(tx, OutboundTx::Bounded(_)));
        tx.try_send(frame()).expect("1st fits");
        tx.try_send(frame()).expect("2nd fits");
        assert!(
            matches!(tx.try_send(frame()), Err(SendError::Full)),
            "a bounded channel must report Full past capacity"
        );
    }
}
