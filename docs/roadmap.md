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

- ~~Per-stream queue-depth override.~~ Shipped (model **(c)**, see
  #28). Each subscription has **its own channel**, sized from the
  stream's manifest `queue_depth` (else the global
  `WSS_MUX_QUEUE_DEPTH`); the writer drains a `tokio_stream::StreamMap`
  of those receivers fairly onto the socket, plus a per-connection
  control channel for closes/connection-level + keep-open `overflow`
  errors. This changed the overflow wire-semantics: a subscriber too
  slow for one stream gets a keep-open `overflow` `error` frame and
  that subscription alone is dropped — the connection and its other
  subscriptions are untouched; the pre-v0.4 `4429` connection-level
  overflow close no longer exists. Per-stream depth is now *truly
  isolated* (no shared per-connection queue; a large per-stream
  `queue_depth` is no longer capped by a connection-wide buffer).
  `queue_depth: 0` / global `WSS_MUX_QUEUE_DEPTH=0` is an explicit
  "unlimited" opt-in (that subscription's channel is unbounded — never
  drops, at the cost of the memory bound). A SIGHUP `queue_depth`
  change resizes existing subscriptions (the channel is recreated;
  frames buffered in the old one are dropped, within the at-most-once
  contract). The initial accounting-based interim (Approach A, #26) was
  superseded by this. See `docs/protocol.md` and `docs/embedding.md`.
- ~~Ed25519 keypair token signing (in addition to HS256).~~ Shipped
  (#29). EdDSA verification is **additive** to HS256: set
  `WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY[_FILE]` alongside or instead of
  the HS256 secret; the token's `alg` selects the trust root. See
  `docs/embedding.md`.
- ~~Optional OIDC token validation as an alternative to signed-handshake
  tokens.~~ Shipped (#31 config + JWKS, #32/#33 validation + ws
  wiring). Opt-in via `WSS_MUX_OIDC_ISSUER` and **mutually exclusive**
  with the handshake key (issuer XOR handshake — configuring both, or
  neither, is a startup error); inert with no issuer set. Validates the
  `auth`-frame JWT against the issuer's JWKS (discovered from the
  issuer or an explicit URL, refreshed by a background task), pins
  `aud`, and maps a configurable groups claim to principals
  (`user:<sub>` plus each prefixed group). See `docs/embedding.md`.

## v0.5

- **Relay coalescing.** When `WSS_MUX_RELAY_COALESCE_MS > 0`, an
  instance batches relayed producer events per peer over that window
  (or `WSS_MUX_RELAY_COALESCE_MAX_EVENTS`, whichever first) and POSTs
  to peers concurrently, raising the per-instance event-volume ceiling.
  `0` (default) or no peers ⇒ byte-identical to pre-v0.5.
- **Ceiling telemetry.** `wss_mux_relay_events_unwanted_total`
  (peer-origin events matching no local subscription — the
  stream-sparsity signal), plus relay flush/queue counters.
- *Subscription persistence was removed from the roadmap:* it
  contradicts the stateless, non-persistent identity in
  `docs/concepts.md`; durability remains the producer's responsibility.
- *Designed but deferred (data-gated on the telemetry above):*
  interest-based selective relay. *Future design consideration:* an
  opt-in inbound overload signal.

## v1.0

- Stable contracts (envelope, manifest, token, wire frames).
- Semver discipline; deprecation cycle for any breaking change.

## Non-goals (any version)

- Bidirectional command transport.
- Built-in message broker.
- Domain payload processing.
- Cross-instance state replication beyond the optional peer-relay
  event fanout.
- TLS termination.
- Token issuance.

## Open questions

- ~~Should batch push responses include per-event status, or stay
  all-or-nothing 204?~~ Resolved in v0.1 as all-or-nothing. Invalid
  schema returns 400, an unknown stream in any event returns 404, and
  on success a single 204 with no per-event detail. Per-event status
  may be reconsidered if observability needs grow.
- ~~Should the manifest support stream wildcards (`chat_*`) for
  audience grants?~~ Resolved in v0.3: an audience entry may end with a
  single trailing `*` (prefix match); a bare `*` admits any
  authenticated connection. A `*` anywhere but the final position is
  rejected at manifest load. See `docs/embedding.md`.
- ~~Should there be a per-stream max-payload-size to prevent
  pathological fanout amplification?~~ Resolved in v0.3: optional
  `max_payload_bytes` per stream, measured as JSON-serialized payload
  length, enforced on push/batch/relay (413, all-or-nothing). See
  `docs/embedding.md`.

These are tracked as GitHub issues once the project is initialized.
