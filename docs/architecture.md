# Architecture

A single binary built around a few small components.

```
         +-----------------+
         |   HTTP server   |  POST /v1/events
         |                 |  POST /v1/events/batch
         +--------+--------+  GET  /healthz
                  |           GET  /readyz
                  v
         +-----------------+
         |   Dispatcher    |  events -> matching subscriptions
         +--------+--------+
                  |
                  v
         +-----------------+
         |    Registry     |  connections + subscriptions
         +--------+--------+
                  ^
                  |
         +-----------------+
         |    WS server    |  /v1/stream
         |                 |  per-connection task
         +-----------------+
```

## Components

### HTTP server

Accepts producer pushes on `POST /v1/events` and the batched
`POST /v1/events/batch`. Bearer-token authenticated against a shared
secret. Validates the event envelope, forwards to the dispatcher.
Stateless.

Also serves `/healthz` (200 if the process is alive) and `/readyz`
(200 once the manifest is loaded and the WS listener is up).

### Dispatcher

Receives validated events from the HTTP server. For each event, looks
up the set of subscriptions matching the event's `stream` (and `key`,
if narrowed) in the registry. For each match it reserves an in-flight
slot against that subscription's send-queue cap (the stream's manifest
`queue_depth`, else the global default), then enqueues a copy of the
event on the connection's shared send queue.

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

Accepts WebSocket connections at `/v1/stream`. For each connection:

- One task reads frames from the client (`auth`, `subscribe`,
  `unsubscribe`).
- A bounded send queue holds outgoing frames (default 1024).
- A second task drains the queue to the socket.

When a subscription exceeds its send-queue depth, the server sends an
`error` frame with `code: overflow` (carrying that subscription's
`id`) and drops only that subscription; the connection stays open.

### Auth

Two pure-function checks:

1. **Token validation** at connection auth time. Signature + TTL.
2. **Audience intersection** at subscribe time. Connection's
   principals ∩ stream's audience.

Neither makes a network call. Both read from the connection's state
plus the manifest.

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

Per connection: `queue_depth × avg_frame_size` bytes (default
1024 × ~1KB ≈ 1MB ceiling). Closed connections release memory
immediately. Setting `WSS_MUX_QUEUE_DEPTH=0` (explicit "unlimited")
removes this ceiling — the per-connection channel becomes unbounded,
so a stalled or non-reading client can grow memory without limit.
That is an opt-in tradeoff for deployments that must never drop
events and trust their consumers to keep up.

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
  behaviour). See `docs/operations.md`.
- **Token issuance.** Only validation.
- **Producer adapters.** No Kafka consumer, no Redis subscriber.
- **TLS termination.** Client-edge TLS is done at the reverse proxy
  (peer-to-peer TLS is a separate, optional axis — see operations).

The default build links one piece of non-Rust code: `ring` (C +
assembly), pulled transitively for crypto (`jsonwebtoken` token
validation since v0.1; `rustls` peer-TLS since v0.3). It needs no
external service to run.
