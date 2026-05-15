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
if narrowed) in the registry. Enqueues a copy of the event on each
matching connection's send queue.

The dispatcher does not block on slow consumers — overflow is the
connection's problem.

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

When the queue fills, the server sends a final `error` frame with
`code: overflow` and closes the connection.

### Auth

Two pure-function checks:

1. **Token validation** at connection auth time. Signature + TTL.
2. **Audience intersection** at subscribe time. Connection's
   principals ∩ stream's audience.

Neither makes a network call. Both read from the connection's state
plus the manifest.

### Manifest

Loaded once at startup from a YAML file. Validated:

- Version `1`.
- Every stream has a non-empty audience.
- No duplicate stream names.

Reloadable on `SIGHUP` (planned for v0.2).

## Concurrency model

- One tokio task per connection per direction (read, write).
- One shared `Arc<Registry>` (using a concurrent map like `DashMap`).
- One shared `Arc<Manifest>` — immutable between reloads.
- HTTP and WebSocket share the listen port (the HTTP server upgrades
  on the WebSocket path).

No locks are held across awaits. No background tasks beyond the
per-connection ones and any optional metric pushers.

## Memory bounds

Per connection: `queue_depth × avg_frame_size` bytes (default
1024 × ~1KB ≈ 1MB ceiling). Closed connections release memory
immediately.

Per stream: O(active subscriptions). Each subscription is a small
struct (≈64 bytes).

Worst-case sanity check: 10k connected clients × 5 subscriptions each
gives 50k entries in the registry plus 10k × 1MB queue ceilings ≈ 10
GB if every client is maximally backlogged. In practice queue
occupancy is near zero outside fanout bursts.

## What's NOT in the binary

- **Storage.** No database, no embedded KV.
- **Service discovery.** No peer awareness; instances are independent.
- **Token issuance.** Only validation.
- **Producer adapters.** No Kafka consumer, no Redis subscriber.
- **TLS termination.** Done at the reverse proxy.

Any of these can be added as Cargo features in later versions if a
specific deployment needs them. The default build is dependency-free.
