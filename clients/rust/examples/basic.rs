//! Minimal subscribe example.
//!
//! Run against a local wss-mux:
//!
//! ```bash
//! cargo run --example basic -- ws://localhost:8080/stream
//! ```

use std::env;

use wss_mux_client::{WssMuxClient, WssMuxError};

#[tokio::main]
async fn main() -> Result<(), WssMuxError> {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "ws://localhost:8080/stream".to_string());

    let client = WssMuxClient::builder()
        .url(url)
        // In real use this would talk to an OIDC provider, a Keycloak
        // adapter, AWS Cognito, etc.
        .get_token(|| async { Ok("your-jwt-here".to_string()) })
        .build()
        .await?;

    let mut sub = client
        .subscribe("notification_banner", Some("room-42"))
        .await?;

    // Publish over the same WebSocket. The connection's principals
    // must intersect the server's `publish` audience for the target
    // stream.
    if let Err(e) = client
        .publish(
            "chat_messages",
            Some("room-42"),
            serde_json::json!({"from": "alice", "text": "hello"}),
        )
        .await
    {
        eprintln!("publish rejected: {e}");
    }

    println!("subscribed; waiting for events (ctrl-c to exit)");

    while let Some(event) = sub.recv().await {
        match event {
            Ok(ev) => println!("{} {:?}: {}", ev.stream, ev.key, ev.payload),
            Err(e) => {
                eprintln!("subscription error: {e}");
                break;
            }
        }
    }

    client.close().await
}
