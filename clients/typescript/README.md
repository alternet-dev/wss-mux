# @wss-mux/client

Browser TypeScript client for the [wss-mux](https://github.com/alternet-dev/wss-mux)
WebSocket multiplexer.

- Zero runtime dependencies.
- Browser-first (uses `globalThis.WebSocket`; injectable for other hosts).
- Automatic reconnect with exponential backoff.
- Token refresh via a caller-provided `getToken` callback on initial connect
  and on close-code `4401` (`expired_token`).
- Typed errors for the documented wss-mux error codes and close codes.

## Install

```bash
npm install @wss-mux/client
```

## Quickstart

```ts
import { WssMuxClient } from "@wss-mux/client";

const client = new WssMuxClient({
  // Whatever path your wss-mux deployment serves the WebSocket on; the
  // handshake response from your auth endpoint is the canonical source.
  wssUrl: "wss://realtime.example.com/stream",
  getToken: async () => kcAuth.token,            // your token source
  onError: (err) => console.warn(err.code, err.message),
});

const sid = await client.subscribe("notification_banner", "*", (event) => {
  console.log(event.payload);
});

// later...
await client.unsubscribe(sid);
await client.close();
```

The `getToken` callback is called on the initial connect and whenever the
server closes the connection with code `4401`. Other reconnects reuse the
cached token.

## API

### `new WssMuxClient(options)`

| Option | Type | Default | Description |
|---|---|---|---|
| `wssUrl` | `string` | required | Full WebSocket URL: `wss://host[:port]/path`. |
| `getToken` | `() => string \| Promise<string>` | required | Returns the auth token. |
| `WebSocket` | `typeof WebSocket` | `globalThis.WebSocket` | Constructor override for non-browser hosts. |
| `reconnect.maxAttempts` | `number` | `Infinity` | Cap on reconnect attempts. |
| `reconnect.initialBackoffMs` | `number` | `1000` | Initial backoff. |
| `reconnect.maxBackoffMs` | `number` | `30000` | Max backoff. |
| `reconnect.backoffMultiplier` | `number` | `2` | Backoff multiplier per attempt. |
| `onError` | `(err: ProtocolError) => void` | — | Called when the server sends an `error` frame. |
| `onStateChange` | `(state: ConnectionState) => void` | — | Called on connection state transitions. |

### `client.subscribe(stream, [key], callback) → Promise<SubscriptionId>`

Binds a subscription. The callback is invoked for every matching event.
Omit `key` to receive all events on the stream.

### `client.unsubscribe(id) → Promise<void>`

Removes a subscription. Idempotent.

### `client.close() → Promise<void>`

Graceful shutdown. After calling `close()`, no further `subscribe()` calls
are accepted.

### `client.connectionState`

Current state: `"idle" | "connecting" | "authenticating" | "ready" |
"reconnecting" | "closing" | "closed"`.

## Error handling

`ProtocolError` wraps server-sent `error` frames. The `code` field is one
of the documented wss-mux error codes (`unauthorized_subscribe`,
`unknown_stream`, `duplicate_subscription_id`, `rate_limited`, `overflow`,
etc.). For per-subscription fatal codes the SDK automatically removes the
subscription from its local map.

`ConnectionClosedError` is surfaced when the server closes with code
`4400` (`bad_frame`), which indicates a protocol bug — the SDK does not
reconnect in that case.

## Development

```bash
npm install
npm run typecheck
npm run build
npm test          # unit tests via node:test
npm run test:e2e  # against a real wss-mux instance; see tests/e2e.test.mjs
```

## License

MIT OR Apache-2.0.
