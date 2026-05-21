# AGENTS

If you're an LLM agent landing in this repo, read this first.

## Mission in one paragraph

`wss-mux` is a small, infrastructure-agnostic WebSocket multiplexer
for server-driven event fanout. Clients connect via WebSocket, present
a signed token, and subscribe to named streams. A producer pushes
events to `wss-mux` via HTTP, and `wss-mux` routes each event to all
matching subscriptions across all connected clients. At-most-once,
ephemeral, no broker, no database, no external infrastructure
required.

## Reading order

1. [README.md](README.md) — pitch + quick start
2. [docs/concepts.md](docs/concepts.md) — first principles + vocabulary
3. [docs/protocol.md](docs/protocol.md) — wire format + state machine
4. [docs/architecture.md](docs/architecture.md) — implementation shape
5. [docs/embedding.md](docs/embedding.md) — integration patterns
6. [docs/operations.md](docs/operations.md) — running it (multi-instance)
7. [docs/roadmap.md](docs/roadmap.md) — versioned plan
8. Source: `src/main.rs` → `src/server/` → `src/auth.rs` +
   `src/oidc.rs` (token validation) → `src/registry.rs` +
   `src/dispatcher.rs` → `src/peers/` (cross-instance relay)

## Key invariants

- `wss-mux` does NOT mint tokens. Only validates — signed-handshake
  (HS256/Ed25519) or, mutually exclusively, against an OIDC issuer's
  JWKS. It never runs or mandates an identity provider.
- `wss-mux` does NOT persist subscriptions or events.
- `wss-mux` does NOT replicate subscription state across instances.
  It performs one-hop, best-effort event relay to discovered peers:
  never gossips state, never multi-hops. Resolving no peers ⇒ no
  relay (byte-identical to single-instance).
- Authentication is two-stage: connection-level (auth frame) and
  subscription-level (audience check).
- Overflow drops the offending subscription (keep-open); it never
  blocks, and backpressure never reaches the producer.
- The default build's only non-Rust code is `ring` (C + assembly),
  pulled transitively for crypto: `jsonwebtoken` (token validation,
  since v0.1; Ed25519 and OIDC JWKS verification, v0.4) and `rustls`
  (peer-relay TLS client since v0.3, also the OIDC JWKS fetch in
  v0.4). That is why the Docker runtime stage is `distroless/cc`. New
  code must not add a *different* native-dependency class — keep
  crypto on the existing `ring` baseline.

## What NOT to do

- **No hard infrastructure coupling in the binary.** No vendor SDKs,
  no Kubernetes/cloud API clients, no broker libraries. Generic,
  overridable conventions are allowed (peer discovery composes a
  DNS-conventional name and reads the ServiceAccount namespace file,
  but is plain DNS, runtime-config-gated, inert without peers, and has
  full non-k8s escape hatches). Deployment-specific patterns belong in
  `docs/operations.md` / `docs/embedding.md`, not hard-coded in `src/`.
- **No silent contract changes.** Wire frame additions, manifest
  schema changes, and token-claim changes go through `docs/protocol.md`
  + roadmap update.
- **No new external-infrastructure dependency.** No broker, database,
  or other service the operator must run. This is the hard line that
  rejected the Redis adapter; peer-relay rides the existing HTTP stack
  + cluster DNS precisely to preserve it. OIDC validation does not
  cross this line: it is opt-in and validates against the operator's
  *existing* identity provider — `wss-mux` mandates no new service.
- **No features that contradict the at-most-once / ephemeral
  contract** without an explicit version bump and roadmap entry.

## Build, test, lint

```bash
cargo build --release
cargo test
cargo clippy --all-targets
cargo bench               # criterion hot-path microbenchmarks
docker build -t wss-mux .
```

Integration tests in `tests/integration/` run pure-Rust — no Docker,
no external services. `benches/` holds the criterion microbenchmarks;
`src/bin/loadgen/` is the `wss-mux-loadgen` load and stress harness
(throughput, latency, connections, peer fleets, traffic oddities).

## Where to put new things

| Kind | Location |
|---|---|
| Wire-format change | `src/envelope.rs` + update `docs/protocol.md` |
| New server route | `src/server/http.rs` or `src/server/ws.rs` |
| Subscription routing | `src/registry.rs` |
| Connection state | `src/connection.rs` |
| Rate limiting | `src/ratelimit.rs` |
| Config knob | `src/config.rs` + README env table |
| Metric | `src/server/metrics.rs` + `docs/architecture.md` |

## Heuristic

The shape that does less is the right shape. Push features outward —
to docs, to the embedder, to optional Cargo features — before adding
them to the core binary.
