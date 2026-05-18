# wss-mux

A small, infrastructure-agnostic WebSocket multiplexer for server-driven
event fanout. Accepts events from a producer, fans them out to
authenticated WebSocket clients that hold multiplexed subscriptions
over a single connection.

```
producer ----push events----> wss-mux <----WebSocket---- clients
                                  |
                                  +- many subscriptions per client
                                  +- per-subscription audience check
                                  +- overflow-close backpressure
```

## Why

A single WebSocket carries one logical channel. Real apps need many:
chat + presence + notifications, dashboards from several feeds, an
ops console with logs and alerts. Doing this naively means one
WebSocket per topic (wasteful, hits browser per-origin limits) or
inventing an ad-hoc multiplexing scheme on every project.

`wss-mux` is the multiplexing scheme, extracted. It also fans an event
out across every subscribed client, so the producer publishes once and
every viewer sees it.

## Core concepts (1-minute version)

- **Connection** — one WebSocket between a client and the mux.
- **Stream** — a named flow of events (e.g. `chat_messages`,
  `presence`). Streams are declared at startup.
- **Subscription** — a connection's binding to a stream (optionally
  narrowed by a key, e.g. one chat room).
- **Event** — a discrete message, produced externally, routed to
  matching subscriptions.
- **Frame** — a protocol message on a connection (`auth`,
  `subscribe`, `unsubscribe`, `event`, `error`).

Each connection multiplexes many subscriptions; each event fans out
across many connections. Two dimensions of multiplexing, one binary.

For depth, see [docs/concepts.md](docs/concepts.md).

## What it is — and isn't

It IS:
- A WebSocket multiplexer with declarative streams.
- A fanout broker for events from a producer (or producers).
- An audience-gated dispatcher (per-stream role check on subscribe).
- An at-most-once, non-replayable, ephemeral event tier.

It IS NOT:
- A message broker. Bring your own event source.
- A persistence layer. No durable subscriptions, no replay.
- An auth server. Bring your own signed-token issuer.
- A bidirectional command transport.
- A presence service. (Add a stream for it and publish to that stream
  yourself.)

## Quick start

From source:

```bash
cargo build --release

# minimum config: shared secrets + a manifest file
WSS_MUX_PUSH_AUTH_TOKEN=<secret> \
WSS_MUX_HANDSHAKE_SIGNING_KEY=<secret> \
WSS_MUX_STREAMS_MANIFEST_PATH=./streams.yaml \
./target/release/wss-mux
```

Or via the prebuilt multi-arch container:

```bash
docker run --rm -p 8080:8080 \
  -e WSS_MUX_PUSH_AUTH_TOKEN=<secret> \
  -e WSS_MUX_HANDSHAKE_SIGNING_KEY=<secret> \
  -e WSS_MUX_STREAMS_MANIFEST_PATH=/etc/wss-mux/streams.yaml \
  -v $(pwd)/streams.yaml:/etc/wss-mux/streams.yaml:ro \
  ghcr.io/alternet-dev/wss-mux:latest
```

`streams.yaml`:

```yaml
version: 1
streams:
  - stream: chat_messages
    audience: [role:member]
  - stream: presence
    audience: [role:member]
```

Publish:

```bash
curl -X POST http://localhost:8080/v1/events \
  -H "Authorization: Bearer <push-token>" \
  -H "Content-Type: application/json" \
  -d '{"stream":"chat_messages","key":"room-42","payload":{"text":"hi"}}'
```

Subscribe (client):

```javascript
const ws = new WebSocket("ws://localhost:8080/v1/stream");
ws.onopen = () =>
  ws.send(JSON.stringify({ type: "auth", token: "<jwt>" }));
ws.onmessage = (m) => {
  const f = JSON.parse(m.data);
  if (f.type === "event") console.log(f.stream, f.key, f.payload);
};
// once auth'd:
ws.send(JSON.stringify({
  type: "subscribe", id: "s1",
  stream: "chat_messages", key: "room-42"
}));
```

## Configuration

All configuration is environment variables, read once at startup.

| Var | Default | Meaning |
|---|---|---|
| `WSS_MUX_PUSH_AUTH_TOKEN` | — (required) | shared bearer secret for `POST /v1/events*` |
| `WSS_MUX_HANDSHAKE_SIGNING_KEY` | — | HS256 client-token secret; optional if an Ed25519 key is set |
| `WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY` | — | Ed25519 (EdDSA) client-token public key, inline PEM |
| `WSS_MUX_HANDSHAKE_ED25519_PUBLIC_KEY_FILE` | — | path to the Ed25519 public-key PEM file (alternative to inline) |
| `WSS_MUX_OIDC_ISSUER` | — | OIDC issuer URL; enables OIDC token validation (mutually exclusive with the handshake keys) |
| `WSS_MUX_OIDC_AUDIENCE` | — | required when OIDC is enabled: the `aud` tokens must carry |
| `WSS_MUX_OIDC_JWKS_URL` | — | explicit JWKS endpoint; absent ⇒ discovered from the issuer |
| `WSS_MUX_OIDC_GROUPS_CLAIM` | `groups` | array claim mapped to principals |
| `WSS_MUX_OIDC_PRINCIPAL_PREFIX` | — | prefix applied to each group value (e.g. `role:`) |
| `WSS_MUX_OIDC_JWKS_REFRESH` | `300` | JWKS re-fetch interval (seconds) |
| `WSS_MUX_STREAMS_MANIFEST_PATH` | — (required) | path to the YAML stream manifest |
| `WSS_MUX_LISTEN_ADDR` | `0.0.0.0:8080` | bind address for HTTP + WebSocket |
| `WSS_MUX_QUEUE_DEPTH` | `1024` | default per-subscription send-queue depth; `0` = unlimited; per-stream `queue_depth` overrides |
| `WSS_MUX_ENVELOPE_STREAM_PATH` | `stream` | dotted path to the stream name in the push body |
| `WSS_MUX_ENVELOPE_KEY_PATH` | `key` | dotted path to the optional key |
| `WSS_MUX_ENVELOPE_PAYLOAD_PATH` | `payload` | dotted path to the payload |
| `WSS_MUX_INBOUND_RATE` | `50` | inbound client-frame rate (frames/sec/conn); `0` disables |
| `WSS_MUX_INBOUND_BURST` | `100` | inbound token-bucket capacity (burst) |
| `WSS_MUX_PEER_RELAY` | (on) | `off` disables cross-instance relay + discovery |
| `WSS_MUX_PEER_SERVICE` | `wss-mux-headless` | headless Service name for the composed discovery FQDN |
| `WSS_MUX_PEER_NAMESPACE` | (auto) | namespace override; else auto-read from the ServiceAccount |
| `WSS_MUX_CLUSTER_DOMAIN` | `cluster.local` | cluster DNS domain |
| `WSS_MUX_PEER_DNS` | — | full peer FQDN override (skips composition) |
| `WSS_MUX_PEERS` | — | static peer CSV for a fixed fleet (no DNS) |
| `WSS_MUX_PEER_DNS_REFRESH_SECS` | `3` | peer re-resolution interval |
| `WSS_MUX_PEER_RELAY_TIMEOUT_MS` | `500` | per-relay request timeout |
| `WSS_MUX_PEER_CA_FILE` | — | private CA to trust for `https://` peers |
| `WSS_MUX_PEER_CLIENT_CERT` | — | client certificate for peer mutual TLS |
| `WSS_MUX_PEER_CLIENT_KEY` | — | client key for peer mutual TLS |

Operational notes:

- **`SIGHUP`** reloads and re-validates the manifest without dropping
  connections; subscriptions invalidated by the new manifest get an
  `error` frame and are dropped, the connection stays open. A failed
  reload keeps the previous manifest serving.
- **Binary framing.** Clients may negotiate the `wss-mux.v1.cbor`
  subprotocol for CBOR instead of JSON; frame shapes are identical.
- **`GET /metrics`** exposes Prometheus/OpenMetrics counters;
  `/healthz` and `/readyz` are the liveness/readiness probes.
- **Peer-relay.** Run >1 instance and a producer push is relayed once
  to discovered peers so every subscriber sees it, with zero new
  infrastructure. Active by default, inert with no resolvable peers
  (byte-identical to single-instance). The conventional Kubernetes
  deploy needs no peer configuration at all.

The envelope-path and rate-limit knobs are detailed in
[docs/embedding.md](docs/embedding.md); running multiple instances is
covered end-to-end in [docs/operations.md](docs/operations.md).

## Reading order

1. [docs/concepts.md](docs/concepts.md) — first principles + vocabulary
2. [docs/protocol.md](docs/protocol.md) — wire format
3. [docs/embedding.md](docs/embedding.md) — how to integrate
4. [docs/architecture.md](docs/architecture.md) — implementation shape
5. [docs/operations.md](docs/operations.md) — running it (incl. multi-instance)
6. [docs/roadmap.md](docs/roadmap.md) — what's in each version

LLM agents: start with [AGENTS.md](AGENTS.md).

## Status

v0.x — pre-stable. Contracts may break between minor versions until
v1.0.

## License

Dual MIT / Apache-2.0.
