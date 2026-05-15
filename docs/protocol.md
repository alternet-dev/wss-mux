# Protocol

The wss-mux protocol runs over a single WebSocket connection. Every
frame is a JSON object with a `type` field. Frames are sent as
WebSocket text messages, one frame per message.

## Frame types

### `auth` (client → server)

First frame on every connection. The server discards any other frame
type until auth succeeds.

```json
{ "type": "auth", "token": "<signed-token>" }
```

The server validates the signature and TTL of the token.

- **Success**: connection enters the authenticated state. No ack frame
  is sent — the absence of a `close` is the ack.
- **Failure**: connection is closed with WebSocket close code `4401`
  ("unauthenticated").

### `subscribe` (client → server)

Binds a subscription within the connection.

```json
{
  "type": "subscribe",
  "id": "sub-1",
  "stream": "chat_messages",
  "key": "room-42"
}
```

Fields:

- `id` — client-chosen subscription identifier, unique within the
  connection. Echoed back on every event delivered to this
  subscription so the client can demultiplex.
- `stream` — the stream name. Must exist in the manifest.
- `key` — optional. Omit to subscribe to all events on the stream;
  provide to narrow to one specific key value.

The server checks the connection's principals against the stream's
audience.

- **Success**: subscription is active. No ack frame.
- **Failure**: an `error` frame is returned with the appropriate code.
  No subscription is created. The connection stays open.

### `unsubscribe` (client → server)

Removes a subscription.

```json
{ "type": "unsubscribe", "id": "sub-1" }
```

Idempotent. No error if the subscription doesn't exist.

### `event` (server → client)

Delivers an event to a matching subscription.

```json
{
  "type": "event",
  "id": "sub-1",
  "stream": "chat_messages",
  "key": "room-42",
  "payload": { "from": "alice", "text": "hello" }
}
```

`id` echoes the client's chosen subscription identifier. `payload` is
forwarded verbatim from the producer — `wss-mux` never inspects or
modifies it.

### `error` (server → client)

Reports a problem.

```json
{
  "type": "error",
  "code": "unauthorized_subscribe",
  "message": "principals do not intersect stream audience",
  "id": "sub-1"
}
```

`id` is included when the error pertains to a specific frame, absent
otherwise.

## Error codes

| Code | When | Closes connection? |
|---|---|---|
| `unknown_frame_type` | client sent an unrecognized `type` | yes |
| `bad_frame` | frame failed JSON parsing or schema | yes |
| `unauthenticated` | non-auth frame before auth, or auth failed | yes |
| `expired_token` | auth token's `exp` claim has passed | yes |
| `unknown_stream` | subscribe to a stream not in the manifest | no |
| `unauthorized_subscribe` | principals do not intersect stream audience | no |
| `duplicate_subscription_id` | subscribe with an `id` already in use | no |
| `overflow` | per-connection send queue overflowed | yes |

Errors that close the connection use WebSocket close code `4xxx`
matching the error semantically (`4400` bad frame, `4401`
unauthenticated, `4429` overflow).

## Connection lifecycle

```
        [WS upgrade]
             |
             v
      [awaiting auth] ----[non-auth frame, or bad token]----> [closed]
             |
             | valid auth
             v
      [authenticated, 0 subs] <----------------+
             |                                  |
             | subscribe (admit)                | unsubscribe
             v                                  |
      [authenticated, N subs] -----------------+
             |
             | overflow / token expiry / client close / shutdown
             v
          [closed]
```

A connection in `awaiting auth` that receives any non-auth frame is
closed immediately. A connection in `authenticated` may hold zero or
more subscriptions; transitions in either direction are free.

## Token expiry behavior

The server checks the `exp` claim only at auth time. It does not
re-validate during the connection. The token's TTL effectively caps
how long any single connection can live before requiring a fresh
token.

Embedders should choose TTL with that tradeoff in mind:

- **Short TTL** (e.g. 1 minute): low replay window, frequent
  re-auth-and-reconnect. Good for sensitive deployments.
- **Long TTL** (e.g. 1 hour): stable connections, longer replay
  window if a token is leaked. Good for typical apps.

5 minutes is a reasonable default.

## Wire format

- Text WebSocket frames containing UTF-8 JSON.
- One protocol frame per WebSocket message.
- WebSocket compression (`permessage-deflate`) is supported and
  recommended for production.
- Binary frames are reserved for future use and currently rejected
  with `bad_frame`.

## Subprotocol negotiation

Clients SHOULD send `Sec-WebSocket-Protocol: wss-mux.v1` on the
upgrade request. The server selects the highest version it supports.
Servers MUST reject upgrades that don't negotiate a compatible
subprotocol.

## Forward compatibility

Until v1.0, the protocol may break between minor versions. Each
breaking change increments the subprotocol version (`wss-mux.v2`,
etc.) so servers can serve old and new clients in parallel during
migration.

At v1.0 the protocol becomes stable. Subsequent additions follow
these rules:

- **Adding a new frame type**: minor version. Old clients ignore
  unknown types if marked optional.
- **Adding a field to an existing frame**: minor version. Old
  clients ignore unknown fields.
- **Adding a new error code**: minor version.
- **Changing or removing anything**: major version.
