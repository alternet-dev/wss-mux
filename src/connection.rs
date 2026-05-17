use tokio::sync::mpsc;

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
