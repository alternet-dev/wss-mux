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
POST /events
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
POST /events/batch
Authorization: Bearer <push-token>

{ "events": [ {...}, {...}, ... ] }
```

Responses:

- `204 No Content` on success.
- `401 Unauthorized` on bad auth.
- `400 Bad Request` on schema failure.
- `413 Payload Too Large` if any event exceeds its stream's
  `max_payload_bytes` cap. In a batch this rejects the whole batch
  (all-or-nothing); nothing is dispatched.

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

Signed with **HS256** using `WSS_MUX_HANDSHAKE_SIGNING_KEY` (a shared
secret between your auth service and `wss-mux`) **or Ed25519 (EdDSA)**,
where `wss-mux` holds only the public key:

- `WSS_MUX_HANDSHAKE_SIGNING_KEY` — HS256 shared secret.
- `WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY` — Ed25519 public key, inline
  PEM; or `WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY_FILE` — path to a PEM
  file (asymmetric: the private signing key never leaves your issuer —
  good for key rotation and least-privilege).

At least one must be set; setting both lets you migrate issuers with
zero downtime (old HS256 clients and new Ed25519 clients are both
accepted during the cutover). The accepted algorithm is **pinned to
the configured key** — a token's `alg` header only selects *which*
configured key to verify against, it can never widen what's accepted,
so an attacker who knows the Ed25519 public key cannot replay it as an
HS256 secret. A bad key fails startup loudly rather than rejecting
every connection at runtime.

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
    "url": "wss://.../stream"
  }
```

`wss-mux` itself never mints tokens. It only validates them.

### OIDC validation (alternative to the handshake token)

Instead of a shared/asymmetric handshake key, `wss-mux` can validate
the `auth`-frame token directly against an OIDC identity provider —
useful when your clients already carry IdP-issued JWTs. It is
**opt-in and inert by default**: with `WSS_MUX_OIDC_ISSUER` unset
nothing changes and no outbound calls are made. Set it and OIDC
becomes the *only* token authority — it is **mutually exclusive** with
the handshake key (one trust root; configuring both is a startup
error), exactly as v0.3's peer-relay treats the cluster's DNS: the IdP
is the embedder's existing infrastructure, not something `wss-mux`
runs.

```bash
WSS_MUX_OIDC_ISSUER=https://idp.example.com   # enables OIDC
WSS_MUX_OIDC_AUDIENCE=wss-mux                  # required: the aud to pin
# optional:
WSS_MUX_OIDC_JWKS_URL=...                      # else discovered from the issuer
WSS_MUX_OIDC_GROUPS_CLAIM=groups               # default
WSS_MUX_OIDC_PRINCIPAL_PREFIX=role:            # default: none
WSS_MUX_OIDC_JWKS_REFRESH=300                  # seconds, default
```

- **Validation.** Signature by `kid` against the JWKS (algorithm
  taken from the JWK and pinned — the token's own `alg` header never
  widens what is accepted), plus `iss`, `aud` (mandatory — pinning an
  audience is required; accepting any `aud` would admit tokens minted
  for another relying party), and `exp`/`nbf`.
- **Principal mapping.** Always `user:<sub>`, plus every value of the
  configured groups claim with the configured prefix (e.g. IdP group
  `members` + prefix `role:` → principal `role:members`, which then
  flows through the unchanged manifest audience checks). A
  missing/empty groups claim yields just `user:<sub>` — a
  low-privilege set, **not** an error (parity with a handshake token
  that carries no principals).
- **JWKS endpoint.** `WSS_MUX_OIDC_JWKS_URL` if set, else discovered
  from `<issuer>/.well-known/openid-configuration`.
- **Resilience.** The JWKS is fetched at startup and refreshed every
  `WSS_MUX_OIDC_JWKS_REFRESH` seconds; a failed refresh is logged +
  metered (`wss_mux_oidc_jwks_refresh`) and keeps the last-good cache,
  so an IdP blip does not 401 every client. Until the *first*
  successful fetch, `/ready` returns 503 (`wss_mux_oidc_jwks_keys`
  gauge stays 0) so the pod is kept out of rotation rather than
  rejecting everyone.

`wss-mux` never contacts the IdP per request — only the periodic JWKS
poll — and adds no new dependency to do it.

## 3. Declaring streams and audiences

`wss-mux` loads a manifest at startup. It declares which streams
exist and which roles may subscribe to each.

```yaml
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
  - stream: presence
    subscribe: [role:member]
  - stream: ops_logs
    subscribe: [role:operator]
```

Provide the file path via `WSS_MUX_STREAMS_MANIFEST_PATH`.

In practice this file is generated from your application's source of
truth (RBAC config, code annotations, etc.) and shipped to `wss-mux`
as part of deployment.

### Audience matching

Each audience entry is matched against the connection's principals in
one of three ways:

| Entry form | Meaning | Example |
|---|---|---|
| exact | principal equals the entry | `role:member` |
| `prefix*` | a principal starts with `prefix` (trailing `*` only) | `tenant:*` admits `tenant:acme` |
| `*` | any authenticated connection, even with no principals | "public" stream |

```yaml
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member, role:operator]
  - stream: tenant_events
    subscribe: [tenant:*]            # any tenant principal
  - stream: status_page
    subscribe: ["*"]                 # public to any authenticated client
```

A `*` is only valid as a single trailing wildcard (or the bare `*`).
Any other use — `*foo`, `ro*le`, `a*b*` — is rejected when the
manifest is loaded (or hot-reloaded), so a malformed grant fails fast
rather than silently denying. Prefix wildcards still require a matching
principal; only the bare `*` is unconditional.

Subscribe-time behavior:

- Stream not in the manifest → `unknown_stream` error.
- No audience entry admits the principals → `unauthorized_subscribe`
  error.

### Publish audience (WS publish)

The `subscribe` field above governs *read*: who may `subscribe` to
this stream. For *write* — `publish` over an open WebSocket — declare
a parallel `publish` field. Same matching rules as `subscribe`;
default empty, which means **no one may WS-publish to this stream**
until you opt in.

```yaml
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    publish:   [role:member]      # members may both subscribe and publish
  - stream: ops_logs
    subscribe: [role:operator]
    publish:   []                  # operators read; nobody WS-publishes
                                   # (HTTP POST /events still works if
                                   #  the push token is configured)
```

HTTP `POST /events` is unaffected by `publish` — it authenticates via
the shared `WSS_MUX_PUSH_AUTH_TOKEN`, not principals. `publish` gates
the WS `publish` frame only.

Publish-time behavior:

- Stream not in the manifest → `unknown_stream` error.
- No `publish` entry admits the principals → `unauthorized_publish`.
- Payload exceeds the stream's `max_payload_bytes` cap →
  `publish_payload_too_large`.
- Per-source publish rate limit exceeded → `rate_limited`.

All are keep-open errors — a rejected publish does not close the
connection.

### Per-source publish rate limit

A token-bucket rate limit can be applied to WS publish frames,
keyed per source (so two WS sessions from the same user share one
budget rather than each getting their own). Configured via env vars,
inert when unset:

| Env var | Default | Meaning |
|---|---|---|
| `WSS_MUX_WS_PUBLISH_RATE` | `0` | Refill rate per source (frames/sec). `0` disables the feature — no limiter is constructed, no idle sweep runs. |
| `WSS_MUX_WS_PUBLISH_BURST` | `2 × RATE` | Token-bucket capacity per source. Defaults to twice the rate when rate is set. |
| `WSS_MUX_WS_PUBLISH_REQUIRE_SUB` | `false` | When `true`, a token whose `sub` is empty is refused at publish time with `unauthorized_publish`. When `false`, such a connection falls back to a per-connection `conn:<id>` bucket. |
| `WSS_MUX_WS_PUBLISH_IDLE_TTL_SECS` | `300` | A source's bucket is dropped after this long without activity. The sweep runs at one-tenth this cadence. |

Source identification, in order:

1. **JWT `sub` non-empty** ⇒ source key is `sub:<sub>`. Two WS sessions
   with the same `sub` (e.g. one user across two devices) share one
   bucket — the rate limit is per-user, not per-connection.
2. **JWT `sub` empty AND `require_sub=false`** ⇒ source key is
   `conn:<id>`, scoped to that one WS session. Coarser but the publish
   proceeds.
3. **JWT `sub` empty AND `require_sub=true`** ⇒ the publish is refused
   with `unauthorized_publish` (keep-open).

The limiter does **not** apply to HTTP `POST /events` — that path
keeps using the shared `WSS_MUX_PUSH_AUTH_TOKEN` and has no
per-source notion (yet). Metric: `wss_mux_ws_publish_rate_limited`.

### Per-stream payload cap

A stream may declare an optional `max_payload_bytes`. A push whose
payload exceeds it is rejected with `413` before any dispatch or
relay:

```yaml
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    max_payload_bytes: 16384      # 16 KiB, guards fanout amplification
```

- The size is the length of the **JSON serialization of the payload**,
  so the cap is stable regardless of the producer/relay wire codec
  (a CBOR-relayed event is measured the same as a JSON push).
- Absent ⇒ no cap. `0` is rejected at manifest load (it would
  black-hole the stream).
- Enforced on `/events`, `/events/batch`, and the internal
  relay path alike — defense-in-depth, so a rolling deploy with mixed
  manifests can't let an oversized event through a not-yet-updated
  instance.
- Batch semantics are all-or-nothing: one oversized event rejects the
  whole batch with `413`; nothing is dispatched.

### Per-stream queue depth

Each subscription has an in-flight send-queue cap. It resolves as: the
stream's manifest `queue_depth` if set, else the global
`WSS_MUX_QUEUE_DEPTH` (default `1024`).

```yaml
version: 1
streams:
  - stream: chat_messages
    subscribe: [role:member]
    queue_depth: 64               # this stream's subscribers are bursty-tolerant
  - stream: audit_log
    subscribe: [role:admin]
    queue_depth: 0                # never drop audit events for slowness
```

- Scope is **per subscription**, not per connection: a connection
  with several subscriptions gets the cap independently for each.
- When a subscription exceeds its cap (a consumer too slow for the
  stream's event rate), only that subscription is dropped: the client
  gets an `error` frame with `code: "overflow"` and the subscription
  `id`, and the connection plus its other subscriptions keep running.
  The client can `subscribe` again to resume. A slow consumer on one
  stream no longer tears down the whole connection.
- Absent ⇒ the global default.
- Each subscription has **its own channel** (no shared per-connection
  queue), so a per-stream `queue_depth` is fully effective — it is
  *not* capped by a connection-wide buffer. A burst on one stream
  cannot evict another stream's backlog on the same connection.
- **`0` means unlimited** (explicit opt-in), at either scope — a
  literal 0-depth queue would be useless, so `0` is the "no cap"
  signal rather than a load error:
  - Per-stream `queue_depth: 0` ⇒ that subscription's channel is
    unbounded; it is never overflow-dropped for depth.
  - Global `WSS_MUX_QUEUE_DEPTH=0` ⇒ every subscription that doesn't
    set its own `queue_depth` gets an unbounded channel. Nothing is
    dropped for backpressure. ⚠️ This removes the memory bound: one
    stalled or non-reading subscription can grow memory without limit
    and OOM the instance. Use it only when consumers are trusted to
    keep up (or bounded by other means). The bounded default exists
    for this reason.
- **SIGHUP applies to existing subscriptions.** A reload that changes
  a stream's `queue_depth` recreates the channel of every live
  subscription on that stream at the new size; frames buffered in the
  old channel at the instant of the swap are dropped (within the
  at-most-once contract — SIGHUP is a rare operator action).
- Tune it per stream: raise it for high-rate streams whose clients
  tolerate bursts, lower it to shed load faster on streams where
  staleness is worse than a gap, set `0` where dropping is never
  acceptable and you accept the memory tradeoff.

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

Multiple `wss-mux` processes, each with its own connected clients. The
producer still pushes each event **once**, to any single instance:
that instance dispatches to its own subscribers and relays the event
to its discovered peers, so clients on every instance see it. This is
**peer-relay** — built in, no producer-side broadcast, no external
infrastructure. The producer contract is identical to the
single-instance case.

Running a fleet is therefore an operations concern, not an embedding
one. [docs/operations.md](operations.md) covers it end to end:
peer-relay, the zero-config Kubernetes deployment, discovery and TLS
knobs, autoscaling, the non-Kubernetes fixed-fleet setup, and the
event-volume ceiling.

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

## Traffic anomalies

How `wss-mux` degrades under slow consumers, oversized payloads,
inbound floods, reconnect storms, and dead peers — what you observe and
the embedder-side fix for each — is documented in the traffic-oddities
runbook in [docs/operations.md](operations.md).
