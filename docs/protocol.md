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

The server validates the signature and TTL of the token. The token is
a JWT signed with **HS256** or **Ed25519 (EdDSA)**; the accepted
algorithm is pinned to the server's configured key (the token's `alg`
only selects which configured key, never widening what is accepted).
See `docs/embedding.md` for key configuration.

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
audience. A principal is admitted if it satisfies any audience entry:
an exact match, a trailing-`*` prefix match (`role:*` admits
`role:member`), or the bare `*` which admits any authenticated
connection (even one whose token carries no principals). A `*` anywhere
but a single trailing position is rejected at manifest load.

- **Success**: subscription is active. No ack frame.
- **Failure**: an `error` frame is returned with the appropriate code.
  No subscription is created. The connection stays open.

### `unsubscribe` (client → server)

Removes a subscription.

```json
{ "type": "unsubscribe", "id": "sub-1" }
```

Idempotent. No error if the subscription doesn't exist.

### `publish` (client → server)

WS-side publish: a connected client emits an event without dropping
back to HTTP `POST /events`. Useful for client-side use cases that
already have a WebSocket open (chat send, presence beacons, collab
edits) — the publish shares the connection with subscribe rather
than opening a parallel HTTP path.

```json
{
  "type": "publish",
  "id": "pub-1",
  "stream": "chat_messages",
  "key": "room-42",
  "payload": { "from": "alice", "text": "hello" }
}
```

Fields:

- `id` — client-chosen, opaque correlation identifier. Echoed back on
  any `error` frame so the client can match a failure to a specific
  publish call.
- `stream` — stream name. Must exist in the manifest.
- `key` — optional. Same semantics as `subscribe` and HTTP push.
- `payload` — forwarded verbatim, identical to HTTP push's payload.

The server checks the connection's principals against the stream's
`publish` audience (separate from the read-side `subscribe` audience).
Same matching rules: exact, trailing-`*` prefix, or bare `*`. Default-
deny when `publish` is absent from the manifest — opt-in per stream.

- **Success**: dispatched to local and peer subscribers. No ack frame —
  the publisher's own subscription (if any matches) receives the
  event through the normal event-frame path.
- **Failure**: an `error` frame is returned with `id` echoed. The
  connection stays open. Possible codes:
  - `unauthorized_publish` — principals do not intersect the
    stream's `publish` audience.
  - `unknown_stream` — stream not in the manifest.
  - `publish_payload_too_large` — payload exceeds the stream's
    `max_payload_bytes` (same cap HTTP push enforces).
  - `rate_limited` — per-source publish rate limit exceeded.

The publish path shares the same downstream fanout as HTTP `POST /events`
— same registry matching, same per-subscription queue, same relay
coalescing, same peer fanout. Source identity for the per-source rate
limit is the JWT's `sub` claim from the connection's auth frame.

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
| `unknown_stream` | subscribe or publish to a stream not in the manifest | no |
| `unauthorized_subscribe` | principals do not intersect stream audience | no |
| `unauthorized_publish` | principals do not intersect stream `publish` audience | no |
| `duplicate_subscription_id` | subscribe with an `id` already in use | no |
| `publish_payload_too_large` | publish payload exceeds the stream's `max_payload_bytes` | no |
| `rate_limited` | inbound frame rate limit exceeded | no |
| `overflow` | a subscription exceeded its send-queue depth | no |

Errors that close the connection use WebSocket close code `4xxx`
matching the error semantically (`4400` bad frame, `4401`
unauthenticated).

`rate_limited` is keep-open: the offending frame is dropped (not
processed), an `error` frame is returned (echoing the frame's `id`
when it has one), and the connection stays usable. The limiter is a
per-connection token bucket; see `WSS_MUX_INBOUND_RATE` /
`WSS_MUX_INBOUND_BURST` in `docs/embedding.md`. Rate limiting is off
when `WSS_MUX_INBOUND_RATE` is `0`.

`overflow` is keep-open and **per-subscription**. Each subscription
has an in-flight send-queue cap: the stream's manifest `queue_depth`
if set, otherwise the global `WSS_MUX_QUEUE_DEPTH` default. When a
subscription exceeds its cap (a consumer too slow for that stream's
event rate), only that subscription is dropped — an `error` frame
with `code: "overflow"` and the subscription's `id` is sent, the
subscription is removed server-side, and the connection and its other
subscriptions continue. A client that still wants the stream simply
`subscribe`s again. Before v0.4 a single slow subscription closed the
whole connection with WebSocket code `4429`; that connection-level
overflow close no longer exists.

A `queue_depth` of `0` at either scope means **unlimited** (explicit
opt-in): that subscription is never overflow-dropped for depth. A
global `WSS_MUX_QUEUE_DEPTH=0` makes the per-connection buffer
unbounded — no `overflow` is ever emitted, at the cost of the
per-connection memory bound (a stalled client can OOM the instance).
See `docs/embedding.md`.

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
             | token expiry / client close / shutdown
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

- One protocol frame per WebSocket message.
- WebSocket compression (`permessage-deflate`) is supported and
  recommended for production.
- The encoding is fixed per connection by the negotiated subprotocol
  (see below): JSON over text frames, or CBOR over binary frames.

### Binary framing (CBOR)

A connection that negotiates the `wss-mux.cbor` subprotocol carries
every frame as a CBOR (RFC 8949) binary message instead of UTF-8 JSON
text. The frame *shapes* are identical — same `auth`/`subscribe`/
`unsubscribe`/`event`/`error` fields — only the encoding differs. CBOR
is self-describing, so producers and clients need no shared schema.

The codec is per-connection and fixed at negotiation: on a CBOR
connection a text message is a wire-type mismatch and is rejected with
`bad_frame` (close `4400`); likewise a binary message on a JSON
(`wss-mux`) connection. `bad_frame` vs `unknown_frame_type` stays
distinguishable on both codecs (a well-formed CBOR map with an
unrecognized `type` is `unknown_frame_type`).

## Subprotocol negotiation

Clients send `Sec-WebSocket-Protocol` on the upgrade request with one
or both of:

- `wss-mux` — JSON text framing.
- `wss-mux.cbor` — CBOR binary framing.

If a client offers both, the server selects `wss-mux.cbor`. The
server MUST reject upgrades that don't negotiate a compatible
subprotocol (HTTP `400`).

The subprotocol identifier carries **no version** — wire-protocol
version is implicit in the server release (per the client/server
lockstep model) rather than negotiated. Breaking wire changes between
releases are documented in the CHANGELOG; clients pinned to a server
release are paired with a server that speaks the same wire.

## Forward compatibility

Until v1.0, the protocol may break between minor versions. The
subprotocol identifier (`wss-mux` / `wss-mux.cbor`) stays stable —
each breaking change is communicated by a server release (and matching
client) rather than by bumping the subprotocol string. Clients pinned
to a release version are paired with a server that speaks the matching
wire.

At v1.0 the protocol becomes stable. Subsequent additions follow
these rules:

- **Adding a new frame type**: minor version. Old clients ignore
  unknown types if marked optional.
- **Adding a field to an existing frame**: minor version. Old
  clients ignore unknown fields.
- **Adding a new error code**: minor version.
- **Changing or removing anything**: major version.
