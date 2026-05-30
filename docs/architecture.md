# Architecture

A single binary built around a few small components.

```
         +-----------------+
         |   HTTP server   |  POST /events           (push)
         |                 |  POST /events/batch     (push)
         |                 |  GET  /events/:stream   (SSE read)
         +--------+--------+  GET  /health, /ready
                  |
                  v
         +-----------------+
         |   Dispatcher    |  events -> matching subscriptions
         +--------+--------+
                  |
                  v
         +-----------------+
         |    Registry     |  connections + subscriptions
         +--------+--------+   (WS and SSE bindings)
                  ^
                  |
         +-----------------+
         |    WS server    |  /stream
         |                 |  subscribe + publish + unsubscribe
         |                 |  per-connection task
         +-----------------+
```

**Unified dispatch.** HTTP `POST /events`, WS `publish`, WS `subscribe`,
and SSE `GET /events/:stream` all share the same dispatcher and the
same registry. A WS-published event takes the same code path
downstream as one pushed over HTTP; an SSE consumer registers a
subscription binding indistinguishable from a WS subscriber's. The
four wire surfaces differ only in framing — schema, audience
matching, per-subscription queue, and overflow semantics are one
implementation.

## Components

### HTTP server

Accepts producer pushes on `POST /events` and the batched
`POST /events/batch`. Bearer-token authenticated against a shared
secret. Validates the event envelope, forwards to the dispatcher.
Stateless.

`GET /events/:stream` exposes the same event stream as a
Server-Sent Events response — the consumer story for backends that
don't want a long-lived WebSocket (a separate `presence_svc`, an
admin dashboard, a sidecar). Auth supports a shared bearer
(`WSS_MUX_READ_AUTH_TOKEN`, parallel to the producer token) or a
JWT whose principals intersect the stream's `subscribe` audience
(parallel to WS subscribe). Each SSE response registers a
subscription binding in the registry exactly as a WS subscribe
would; the dispatcher writes to it transparently. See
`docs/embedding.md` for the wire shape and auth matrix.

Also serves `/health` (200 if the process is alive) and `/ready`
(200 once the manifest is loaded and the WS listener is up).

### Dispatcher

Receives validated events from the HTTP server. For each event, looks
up the set of subscriptions matching the event's `stream` (and `key`,
if narrowed) in the registry. Each subscription has its own bounded
channel (sized at subscribe time from the stream's effective
`queue_depth`); the dispatcher sends a copy of the event straight onto
that channel. A full channel means that one subscription is
overflowing — handled per-subscription, with no shared per-connection
queue.

The dispatcher does not block on slow consumers. A subscription that
cannot reserve a slot has overflowed and is dropped on its own — the
connection and its other subscriptions are unaffected.

### Registry

Concurrent map keyed by stream name. Each entry holds the set of
(connection-id, subscription-id, optional-key) bindings.

- Adding/removing a subscription: O(1) amortized.
- Dispatching an event: O(matching subscriptions).

For very large subscription sets, a future version may add a
secondary index keyed by `key` to accelerate keyed-dispatch. v0.1
scans linearly within a stream.

### WS server

Accepts WebSocket connections at `/stream`. For each connection:

- One reader task handles client frames (`auth`, `subscribe`,
  `unsubscribe`, `publish`).
  - On `subscribe`, the reader creates the subscription's channel
    and hands the receiver to the writer.
  - On `publish`, the reader audience-checks the stream's
    `publish` field, applies the per-stream payload cap and the
    optional per-source rate limit, then forwards the event to the
    dispatcher — the same code path HTTP `POST /events` uses.
- Each subscription has its own channel; a per-connection control
  channel carries closes + connection-level / keep-open `overflow`
  errors.
- One writer task fairly merges the control channel and a
  `tokio_stream::StreamMap` of the per-subscription receivers onto the
  socket. A subscription's receiver ends when its sender is dropped
  (unsubscribe / overflow / teardown / SIGHUP cap-recreate), and the
  `StreamMap` evicts it.

When a subscription exceeds its send-queue depth, the server sends an
`error` frame with `code: overflow` (carrying that subscription's
`id`) and drops only that subscription; the connection stays open.
WS publish rejections (`unauthorized_publish`,
`publish_payload_too_large`, `rate_limited`) are likewise keep-open —
a single bad frame does not close the connection.

### Auth

Two pure-function checks:

1. **Token validation** at connection auth time. Signature + TTL.
   Signed-handshake tokens verify against the configured HS256 and/or
   Ed25519 key (`alg`-selected). The mutually-exclusive OIDC mode
   instead verifies the JWT against the issuer's JWKS by `kid` and
   pins `aud`; the JWKS is fetched and refreshed by a background task,
   so this check itself stays local.
2. **Audience intersection** at subscribe time. Connection's
   principals ∩ stream's audience. OIDC maps the token's groups claim
   to those principals; signed-handshake tokens carry them directly.

Neither makes a network call on the request path. Both read from the
connection's state plus the manifest — OIDC additionally from the
background-refreshed JWKS, never an inline fetch.

### Manifest

Loaded at startup from a YAML file. Validated:

- Version `1`.
- Every stream has a non-empty audience.
- No duplicate stream names.

**Hot-reload (`SIGHUP`).** Sending `SIGHUP` re-reads and re-validates the
manifest file. A successful reload atomically swaps the in-memory manifest
(stored behind a `tokio::sync::watch`, so reads stay cheap and connections
get a change signal). A failed reload (missing file, invalid schema) is
logged and metered (`wss_mux_manifest_reloads_total{result="error"}`) but
the previous manifest keeps serving — a bad edit never takes the process
down.

On reload, every connection re-validates its active subscriptions against
the new manifest. A subscription whose stream was removed, or whose
stream's audience no longer intersects the connection's principals, is
revoked: the client receives an `error` frame (`unknown_stream` or
`unauthorized_subscribe`, carrying the subscription `id`), the binding is
dropped from the registry, and `wss_mux_subscriptions_revoked_total` is
incremented. The connection itself stays open — unaffected subscriptions
on it keep working, and the client may re-subscribe.

## Concurrency model

- One tokio task per connection per direction (read, write).
- One shared `Arc<Registry>` (using a concurrent map like `DashMap`).
- One `watch`-held `Arc<Manifest>` — immutable between `SIGHUP` reloads;
  a reload swaps the `Arc` atomically.
- HTTP and WebSocket share the listen port (the HTTP server upgrades
  on the WebSocket path).

No locks are held across awaits. No background tasks beyond the
per-connection ones and any optional metric pushers.

## Memory bounds

Per subscription: `effective_queue_depth × avg_frame_size` bytes
(default 1024 × ~1KB ≈ 1MB per subscription). A connection's ceiling
is therefore the sum over its subscriptions — bounding load-shedding
per stream rather than pooling it. Closed connections (and dropped
subscriptions) release memory immediately. A `queue_depth` of `0`
(per-stream or global "unlimited") makes that subscription's channel
unbounded, so a stalled or non-reading subscription can grow memory
without limit — an opt-in tradeoff for streams that must never drop
and whose consumers are trusted to keep up.

Per stream: O(active subscriptions). Each subscription is a small
struct (≈64 bytes).

Worst-case sanity check: 10k connected clients × 5 subscriptions each
gives 50k entries in the registry plus 10k × 1MB queue ceilings ≈ 10
GB if every client is maximally backlogged. In practice queue
occupancy is near zero outside fanout bursts.

## What's NOT in the binary

- **Storage.** No database, no embedded KV.
- **Replicated state.** Instances share no subscription/connection
  state. The one cross-instance mechanism is peer-relay (v0.3): an
  instance relays a producer push once to its DNS-discovered peers so
  each fans out to its own clients — best-effort, one-hop, no gossip,
  no shared state. Resolving no peers ⇒ inert (single-instance
  behaviour). In v0.5 the relay can optionally coalesce:
  `WSS_MUX_RELAY_COALESCE_MS > 0` batches per-push relays through a
  bounded queue drained by a single **supervised** flush task (a
  panic is metered `wss_mux_relay_flush_restarts_total` and the task
  restarts), POSTing peers concurrently; `0` / no peers ⇒ inert. The
  flush SPOF is bounded to multi-instance deployments only. Fleet
  stream-sparsity is observable as
  `wss_mux_relay_events_unwanted_total` (peer-origin events with no
  local subscriber) over `wss_mux_relay_events_relayed_total`. See
  `docs/operations.md`.
- **Token issuance.** Only validation.
- **Producer adapters.** No Kafka consumer, no Redis subscriber.
- **TLS termination.** Client-edge TLS is done at the reverse proxy
  (peer-to-peer TLS is a separate, optional axis — see operations).

The default build links one piece of non-Rust code: `ring` (C +
assembly), pulled transitively for crypto (`jsonwebtoken` token
validation since v0.1, incl. Ed25519 + OIDC JWKS verification since
v0.4; `rustls` peer-TLS since v0.3, which also carries the OIDC JWKS
fetch). It needs no external service to run — OIDC, when enabled,
validates against the operator's existing identity provider.
