use crate::envelope::ServerFrame;

pub type ConnId = u64;

/// Outbound is what the writer task drains from the per-connection queue.
///
/// `Frame` is a normal server-to-client frame (event, or in PR4 an error
/// frame on a still-open connection). `Close` is the final message on the
/// connection: the writer emits the optional accompanying frame, then sends
/// a WebSocket close with the given code, then exits.
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
