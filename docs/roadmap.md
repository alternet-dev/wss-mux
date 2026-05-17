# Roadmap

Versions are SemVer-flavored. Pre-1.0, minor versions may break
contracts. Stability commitments arrive at 1.0.

## v0.1 — initial

Scope:

- HTTP push endpoint (single + batch).
- WebSocket endpoint with `auth`, `subscribe`, `unsubscribe`, `event`,
  `error` frames.
- HS256-signed handshake tokens.
- YAML stream manifest loaded at startup.
- In-memory connection registry; in-memory per-connection queues.
- Overflow-close backpressure.
- Prometheus metrics; `/healthz` + `/readyz` endpoints.
- Multi-arch Docker image released on tag.
- Integration tests for: push-to-fanout, auth, audience gating,
  overflow, reconnect.

Out of scope:

- Binary frames.
- Manifest hot-reload.
- Token re-validation during connection.
- Adaptive queue depth.
- Subscription persistence.

## v0.2 — shipped

- ~~Manifest hot-reload via `SIGHUP`.~~ Shipped. Reload re-validates
  active subscriptions; revoked ones get an `error` frame and the
  connection stays open.
- ~~Configurable event envelope shape (custom JSON paths for `stream`
  and `key`, so producers don't have to use exactly those field
  names).~~ Shipped. Dotted paths for stream/key/payload via
  `WSS_MUX_ENVELOPE_*_PATH`; defaults reproduce the classic body.
- ~~Per-connection rate limits (token bucket).~~ Shipped. Inbound
  frame token bucket; `WSS_MUX_INBOUND_RATE`/`_BURST`; `rate_limited`
  keep-open error.
- ~~Binary frame opt-in (CBOR or MessagePack payload framing).~~
  Shipped as CBOR via the `wss-mux.v1.cbor` subprotocol; codec is
  per-connection, frame shapes unchanged.

## v0.3

- **Peer-relay cross-instance fanout.** An instance that receives a
  producer push relays it once to its discovered peers; every instance
  fans out to its own clients. Producer write-count is O(1) in the
  instance count and the producer contract is unchanged (still
  `POST /v1/events`). Discovery is DNS-based (a conventional headless
  Service name, composed from an auto-detected namespace) with a
  static-list fallback for non-k8s; zero new external infrastructure
  and zero new external dependency-class. Runtime-config gated (active
  by default, inert with no resolvable peers ⇒ byte-identical to
  single-instance). *Supersedes the previously-planned Redis pubsub
  adapter, which was rejected for imposing external infra contrary to
  the project's infrastructure-agnostic pitch.*
- Stream wildcards in manifest audiences (trailing `*` prefix match).
- Per-stream max-payload-size (manifest cap; oversized push rejected).
- Operational runbooks (`docs/operations.md`).

## v0.4

- Per-stream queue-depth override, implemented as per-subscription send
  queues (replaces the per-connection single queue; changes overflow
  wire-semantics from per-connection close to per-subscription —
  designed as its own spec).
- Ed25519 keypair token signing (in addition to HS256).
- Optional OIDC token validation as an alternative to signed-handshake
  tokens.

## v0.5

- Subscription persistence (Redis-backed) for embedders that want
  client subscriptions to survive `wss-mux` restarts.

## v1.0

- Stable contracts (envelope, manifest, token, wire frames).
- Semver discipline; deprecation cycle for any breaking change.

## Non-goals (any version)

- Bidirectional command transport.
- Built-in message broker.
- Domain payload processing.
- Cross-instance state replication beyond the optional pubsub fanout.
- TLS termination.
- Token issuance.

## Open questions

- ~~Should batch push responses include per-event status, or stay
  all-or-nothing 204?~~ Resolved in v0.1 as all-or-nothing. Invalid
  schema returns 400, an unknown stream in any event returns 404, and
  on success a single 204 with no per-event detail. Per-event status
  may be reconsidered if observability needs grow.
- Should the manifest support stream wildcards (`chat_*`) for
  audience grants?
- Should there be a per-stream max-payload-size to prevent pathological
  fanout amplification?

These are tracked as GitHub issues once the project is initialized.
