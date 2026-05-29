# wss-mux-client

Rust client for the [wss-mux](https://github.com/alternet-dev/wss-mux)
WebSocket multiplexer.

- Minimal dependency surface: `tokio`, `tokio-tungstenite`, `serde`,
  `serde_json`, `thiserror`, `futures-util`.
- Subscribe **and** publish over a single WebSocket connection.
- Automatic reconnect with exponential backoff; subscriptions replay
  on reconnect.
- Async token-source callback, refreshed on close-code `4401`
  (`expired_token`).
- Typed errors for the documented wss-mux error codes.

## Install

```toml
[dependencies]
wss-mux-client = "0.5"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quickstart

```rust,no_run
use wss_mux_client::{WssMuxClient, WssMuxError};

#[tokio::main]
async fn main() -> Result<(), WssMuxError> {
    let client = WssMuxClient::builder()
        .url("wss://realtime.example.com/stream")
        .get_token(|| async { Ok("your-jwt-here".to_string()) })
        .build()
        .await?;

    let mut sub = client.subscribe("notification_banner", Some("room-42")).await?;
    while let Some(event) = sub.recv().await {
        match event {
            Ok(ev) => println!("{:?}", ev.payload),
            Err(e) => {
                eprintln!("subscription error: {e}");
                break;
            }
        }
    }

    client.close().await
}
```

The `get_token` callback is invoked on the initial connect and whenever
the server closes the connection with code `4401`. Other reconnects
reuse the cached token.

## API

### `WssMuxClient::builder()`

Returns a `ClientBuilder`. Required: `.url(...)` and `.get_token(...)`.
Optional:

- `.reconnect(ReconnectOptions { .. })` overrides the default backoff
  (`1s` initial, `30s` cap, `2x` multiplier, no attempt cap).
- `.publish_settle(Duration)` overrides the publish settle window
  (default `250 ms`).

### `client.subscribe(stream, key) -> Subscription`

Binds a subscription. `key` is `Option<&str>` — pass `None` to receive
all events on the stream.

### `client.publish(stream, key, payload) -> ()`

Emits an event on `stream`. Same downstream dispatch path as the
server's HTTP `POST /events` — same per-subscription delivery, same
peer fanout. The connection's principals must intersect the stream's
`publish` audience on the server; otherwise the call rejects with
`WssMuxError::Protocol { code: ErrorCode::UnauthorizedPublish, .. }`.

Ack-by-absence: the server does not send a success frame. The returned
future resolves once the configured publish settle window
(`publish_settle`, default 250 ms) elapses without an error frame
matching the publish's id. Error frames inside the window reject with
the matching `Protocol` error. Connection drops reject with
`WssMuxError::ConnectionClosed`.

### `Subscription::recv() -> Option<Result<EventFrame, WssMuxError>>`

Receives the next event or per-subscription error. Returns `None` when
the subscription is terminated (server-side fatal error, explicit
unsubscribe, or client close).

Per-subscription fatal codes (`unauthorized_subscribe`, `unknown_stream`,
`duplicate_subscription_id`, `overflow`) deliver one `Err(Protocol { .. })`
on the channel and then close it. Other codes (e.g. `rate_limited`) are
keep-open at the wire level and don't terminate the subscription.

Drop on `Subscription` sends an unsubscribe over the wire (best-effort
if the connection has already been torn down).

### `client.close() -> ()`

Graceful shutdown. Sends a close frame, then waits briefly for the
drive task to exit.

## Error handling

`WssMuxError::Protocol { code, message, id }` wraps server-sent error
frames. The `code` field is a typed `ErrorCode` enum covering every
code documented in `docs/protocol.md`.

`WssMuxError::ConnectionClosed { code, reason }` is surfaced when the
server closes with code `4400` (`bad_frame`) — which indicates a
protocol bug on the client side and where the SDK does not reconnect —
or as the rejection reason for a pending `publish()` whose connection
dropped before it could settle.

`WssMuxError::ReconnectExhausted` surfaces when the configured
`max_attempts` cap is hit.

## Development

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

## License

MIT OR Apache-2.0.
