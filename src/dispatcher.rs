use crate::envelope::{EventEnvelope, ServerFrame};
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

    for (conn_id, sub_id) in matches {
        let Some(sender) = state.sender(conn_id) else {
            stats.dropped_closed += 1;
            continue;
        };
        let frame = ServerFrame::Event {
            id: sub_id,
            stream: envelope.stream.clone(),
            key: envelope.key.clone(),
            payload: envelope.payload.clone(),
        };
        match sender.try_send(frame) {
            Ok(()) => stats.delivered += 1,
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => stats.dropped_full += 1,
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => stats.dropped_closed += 1,
        }
    }
    stats
}
