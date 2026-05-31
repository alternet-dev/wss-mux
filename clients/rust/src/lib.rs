//! Rust client for the [wss-mux](https://github.com/alternet-dev/wss-mux)
//! WebSocket multiplexer.
//!
//! ## Quickstart
//!
//! ```no_run
//! # async fn run() -> Result<(), wss_mux_client::WssMuxError> {
//! use wss_mux_client::WssMuxClient;
//!
//! let client = WssMuxClient::builder()
//!     .url("ws://localhost:8080/stream")
//!     .get_token(|| async { Ok("your-jwt-here".to_string()) })
//!     .build()
//!     .await?;
//!
//! let mut sub = client.subscribe("chat_messages", Some("room-42")).await?;
//! while let Some(event) = sub.recv().await {
//!     match event {
//!         Ok(ev) => println!("{:?}", ev.payload),
//!         Err(e) => {
//!             eprintln!("subscription error: {e}");
//!             break;
//!         }
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! The SDK reconnects automatically with exponential backoff and replays
//! active subscriptions on reconnect. The `get_token` callback is invoked
//! on the initial connect and on close-code `4401` (`expired_token`);
//! other reconnects reuse the cached token.

mod client;
mod error;
mod types;

pub use client::{ClientBuilder, ConnectionState, ReconnectOptions, Subscription, WssMuxClient};
pub use error::WssMuxError;
pub use types::{
    ClientFrame, ErrorCode, PublishId, ServerFrame, SubscriptionId, CLOSE_BAD_FRAME, CLOSE_NORMAL,
    CLOSE_UNAUTHENTICATED,
};

/// Event delivered to a subscription. The shape mirrors the
/// `event` frame on the wire.
#[derive(Debug, Clone)]
pub struct EventFrame {
    /// Subscription this event was matched to.
    pub id: SubscriptionId,
    pub stream: String,
    pub key: Option<String>,
    pub payload: serde_json::Value,
}
