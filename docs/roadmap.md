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

## v0.2

- Manifest hot-reload via `SIGHUP`.
- Configurable event envelope shape (custom JSON paths for `stream`
  and `key`, so producers don't have to use exactly those field
  names).
- Per-connection rate limits (token bucket).
- Binary frame opt-in (CBOR or MessagePack payload framing).

## v0.3

- Optional Redis pubsub adapter for cross-instance fanout without
  producer broadcast amplification. Cargo feature; off by default.
- Per-stream queue depth override.

## v0.4

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

- Should batch push responses include per-event status, or stay
  all-or-nothing 204?
- Should the manifest support stream wildcards (`chat_*`) for
  audience grants?
- Should there be a per-stream max-payload-size to prevent pathological
  fanout amplification?

These are tracked as GitHub issues once the project is initialized.
