# Embedding wss-mux

`wss-mux` is infrastructure. Three responsibilities sit with you, the
embedder:

1. **Produce events.** Push to `wss-mux` via HTTP.
2. **Mint client tokens.** Sign tokens with the shared key.
3. **Declare streams and audiences.** Provide the manifest file.

This doc walks through each, plus the common deployment patterns.

## 1. Producing events

After a domain event happens in your application — a chat message
was sent, a feed item was created, a user came online — push it to
`wss-mux`:

```http
POST /v1/events
Authorization: Bearer <push-token>
Content-Type: application/json

{
  "stream": "chat_messages",
  "key": "room-42",
  "payload": { "from": "alice", "text": "hello" }
}
```

The push token is a shared secret configured via
`WSS_MUX_PUSH_AUTH_TOKEN`. Rotate by changing the env var and
restarting.

For batch efficiency:

```http
POST /v1/events/batch
Authorization: Bearer <push-token>

{ "events": [ {...}, {...}, ... ] }
```

Responses:

- `204 No Content` on success.
- `401 Unauthorized` on bad auth.
- `400 Bad Request` on schema failure.

### Custom envelope shape

If your producer already emits events with its own field names, you
don't have to reshape them. Point `wss-mux` at the fields with dotted
paths:

| Env var | Default | Meaning |
|---|---|---|
| `WSS_MUX_ENVELOPE_STREAM_PATH` | `stream` | path to the stream name (required, non-empty string) |
| `WSS_MUX_ENVELOPE_KEY_PATH` | `key` | path to the optional key (string, or absent/`null`) |
| `WSS_MUX_ENVELOPE_PAYLOAD_PATH` | `payload` | path to the payload (any JSON; explicit `null` allowed, absent is an error) |

Paths traverse object fields only (`meta.topic` → `body["meta"]["topic"]`);
there is no array indexing. An empty path resolves to the whole body.
The batch wrapper is always `{"events": [...]}` — only the per-event
shape is configurable, and each element is interpreted with the same
paths.

With, say, `WSS_MUX_ENVELOPE_STREAM_PATH=meta.topic`,
`WSS_MUX_ENVELOPE_KEY_PATH=meta.partition_key`,
`WSS_MUX_ENVELOPE_PAYLOAD_PATH=data`, this body works unchanged:

```json
{
  "meta": { "topic": "chat_messages", "partition_key": "room-42" },
  "data": { "from": "alice", "text": "hello" }
}
```

The defaults reproduce the classic `{stream,key,payload}` body exactly,
so existing producers need no change.

**Pushes are fire-and-forget from the producer's perspective.**
`wss-mux` dispatches immediately; if delivery to a client fails
(overflow, disconnected), the event is silently dropped. This matches
the at-most-once contract.

## 2. Minting client tokens

Your authentication service issues a signed token to each client
before it connects:

```json
{
  "iss": "your-app",
  "iat": 1747008000,
  "exp": 1747008300,
  "sub": "user:abc",
  "principals": ["role:member", "user:abc", "tenant:t1"]
}
```

Signed with HS256 using `WSS_MUX_HANDSHAKE_SIGNING_KEY` (shared
between your auth service and `wss-mux`).

Required claims:

- `iss`, `iat`, `exp` — standard JWT.
- `sub` — opaque actor identifier (used by `wss-mux` only in logs).
- `principals` — array of role/identity strings used in
  subscribe-time audience checks.

TTL recommendation: 3-5 minutes. Short enough to limit replay; long
enough to absorb typical reconnect storms.

The client typically obtains the token via an authenticated endpoint
on your application:

```http
POST /realtime/handshake
Authorization: <your-app-auth>

→ {
    "token": "<jwt>",
    "expires_at": "...",
    "url": "wss://.../v1/stream"
  }
```

`wss-mux` itself never mints tokens. It only validates them.

## 3. Declaring streams and audiences

`wss-mux` loads a manifest at startup. It declares which streams
exist and which roles may subscribe to each.

```yaml
version: 1
streams:
  - stream: chat_messages
    audience: [role:member]
  - stream: presence
    audience: [role:member]
  - stream: ops_logs
    audience: [role:operator]
```

Provide the file path via `WSS_MUX_STREAMS_MANIFEST_PATH`.

In practice this file is generated from your application's source of
truth (RBAC config, code annotations, etc.) and shipped to `wss-mux`
as part of deployment.

Subscribe-time behavior:

- Stream not in the manifest → `unknown_stream` error.
- Principals don't intersect audience → `unauthorized_subscribe`
  error.

## Deployment patterns

`wss-mux` is a single binary that listens on one port for both HTTP
and WebSocket. Deploy it however your environment likes — container,
systemd unit, function runtime with warm instances, plain process,
whatever fits. Specifically out of scope:

- A particular container orchestrator. Kubernetes, Nomad, ECS, plain
  Docker — all fine.
- A particular reverse proxy. Caddy, nginx, HAProxy, Envoy — all
  fine. TLS termination is the proxy's job.
- A particular service discovery mechanism.

### Single-instance

One `wss-mux` process. Producer pushes to it. Clients connect to it
(typically routed through your reverse proxy).

Holds tens of thousands of concurrent connections on modest hardware.
Memory is dominated by per-connection queue ceilings, which are
bounded (default ~1MB each, mostly idle).

This is the right starting point for most deployments.

### Multi-instance

Multiple `wss-mux` processes, each with their own connected clients.
Each event must reach every instance — the producer broadcasts.

Discovery is your choice; common patterns:

- **DNS-based**: deploy `wss-mux` behind a headless DNS record.
  Producer resolves the name to a list of IPs and pushes to each.
- **Service mesh**: configure the mesh to fan-out producer pushes to
  all `wss-mux` endpoints.
- **Static list**: configure the producer with a list of instance
  URLs.
- **Sidecar-per-producer-pod**: each producer instance runs a local
  `wss-mux` sidecar. Producer pushes to `localhost`. Clients hit any
  pod via the load balancer; cross-pod event delivery requires
  producer-to-producer broadcast (out of scope for `wss-mux` to
  solve).

`wss-mux` doesn't pick for you because the right answer depends on
your environment. The OSS roadmap (v0.3) plans an optional Redis
pubsub adapter to reduce broadcast amplification when N gets large.

### Producer-pod count vs `wss-mux`-pod count

For M producer pods and N `wss-mux` pods, each event is M × N pushes
in the worst case (every producer pushes every event to every
`wss-mux`). At small scale (M, N ≤ 5) this is trivial. At large
scale, consider:

- Reducing N via vertical scaling (one big `wss-mux` is usually fine
  up to ~100k connections).
- Using the v0.3 pubsub adapter (producer pushes once to Redis; all
  `wss-mux` instances subscribe).
- A producer-side fanout daemon that absorbs `M-fold` amplification.

## Connection rate limiting

Each connection has an inbound token bucket gating client-sent frames
(`auth`/`subscribe`/`unsubscribe`) — protection against subscribe
storms and malformed-frame floods.

| Env var | Default | Meaning |
|---|---|---|
| `WSS_MUX_INBOUND_RATE` | `50` | sustained frames/sec/connection; `0` disables the limiter |
| `WSS_MUX_INBOUND_BURST` | `100` | bucket capacity — largest instantaneous burst before the sustained rate applies |

A throttled frame is **dropped, not processed**, and the client gets a
keep-open `rate_limited` error frame (echoing the frame's `id` when it
has one). The connection stays usable; once the bucket refills, frames
flow again. Rejections increment `wss_mux_frames_rate_limited_total`.

The bucket starts full, so a client can send up to `burst` frames
immediately (a normal `auth` + a handful of `subscribe`s is well within
the defaults). Tune `RATE` to your steady subscribe/unsubscribe churn
and `BURST` to the largest legitimate reconnect-resubscribe spike.

## Tradeoffs you should know

- **Restarts kill connections.** `wss-mux` is stateless about
  subscriptions; clients reconnect and re-subscribe. If sub-second
  reconnect blips are acceptable, this is fine. If not, you need
  durable subscriptions, which is out of scope here.
- **No replay.** Disconnected clients miss events. Your application
  must provide a reconciliation API (e.g. `GET /messages?since=...`)
  for clients to catch up.
- **Producer is responsible for delivery semantics beyond at-most-
  once.** If you need at-least-once, dual-write: persist to a
  durable log and push to `wss-mux`. The durable log feeds
  reconciliation; `wss-mux` provides the live feed.
