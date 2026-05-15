# AGENTS

If you're an LLM agent landing in this repo, read this first.

## Mission in one paragraph

`wss-mux` is a small, infrastructure-agnostic WebSocket multiplexer
for server-driven event fanout. Clients connect via WebSocket, present
a signed token, and subscribe to named streams. A producer pushes
events to `wss-mux` via HTTP, and `wss-mux` routes each event to all
matching subscriptions across all connected clients. At-most-once,
ephemeral, no broker, no database, no required dependencies.

## Reading order

1. [README.md](README.md) — pitch + quick start
2. [docs/concepts.md](docs/concepts.md) — first principles + vocabulary
3. [docs/protocol.md](docs/protocol.md) — wire format + state machine
4. [docs/architecture.md](docs/architecture.md) — implementation shape
5. [docs/embedding.md](docs/embedding.md) — integration patterns
6. [docs/roadmap.md](docs/roadmap.md) — versioned plan
7. Source: `src/main.rs` → `src/server/` → `src/registry.rs` +
   `src/subscriptions.rs`

## Key invariants

- `wss-mux` does NOT mint tokens. Only validates.
- `wss-mux` does NOT persist subscriptions or events.
- `wss-mux` does NOT broadcast to peer instances. Multi-instance
  fanout is the producer's job.
- Authentication is two-stage: connection-level (auth frame) and
  subscription-level (audience check).
- Overflow → close, not block. Backpressure never reaches the
  producer.
- Default build has zero non-Rust dependencies.

## What NOT to do

- **No infrastructure assumptions in the binary.** No
  Kubernetes-specific, Caddy-specific, or vendor-specific code.
  Patterns belong in `docs/embedding.md`, not in `src/`.
- **No silent contract changes.** Wire frame additions, manifest
  schema changes, and token-claim changes go through `docs/protocol.md`
  + roadmap update.
- **No native library deps in the default build.** Optional features
  may pull native libs but must be off by default.
- **No features that contradict the at-most-once / ephemeral
  contract** without an explicit version bump and roadmap entry.

## Build, test, lint

```bash
cargo build --release
cargo test
cargo clippy --all-targets
docker build -t wss-mux .
```

Integration tests in `tests/integration/` run pure-Rust — no Docker,
no external services.

## Where to put new things

| Kind | Location |
|---|---|
| Wire-format change | `src/envelope.rs` + update `docs/protocol.md` |
| New server route | `src/server/http.rs` or `src/server/ws.rs` |
| Subscription routing | `src/subscriptions.rs` |
| Connection state | `src/connection.rs` |
| Config knob | `src/config.rs` + README env table |
| Metric | `src/server/metrics.rs` + `docs/architecture.md` |

## Heuristic

The shape that does less is the right shape. Push features outward —
to docs, to the embedder, to optional Cargo features — before adding
them to the core binary.
