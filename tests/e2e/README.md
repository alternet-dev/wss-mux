# wss-mux SDK e2e harness

End-to-end verification that the Rust and TypeScript SDKs and the
server agree on the wire protocol. Boots a real `wss-mux` binary
with the test manifest, then runs both SDKs' e2e suites against it.

```bash
tests/e2e/run.sh           # rust + typescript
tests/e2e/run.sh rust      # rust only
tests/e2e/run.sh ts        # typescript only
```

Locally takes ~5 s including server boot. On a cold GitHub Actions
runner, ~30 s.

## What's covered

For each SDK:

- Connect, authenticate via JWT, subscribe with `key=`.
- Publish over the WS and observe the event back through the
  publisher's own subscribe channel (self-roundtrip; proves
  registry+dispatch+fanout).
- Publish to a `readonly` stream and receive
  `Protocol(unauthorized_publish)` — proves audience gating.
- Push via HTTP `POST /events` from the harness and observe via WS
  subscribe — proves cross-transport delivery.
- TypeScript cross-client: one SDK instance publishes, another
  observes — proves the dispatcher routes to other connections, not
  only the publisher's own.

## Configuration

The harness mints a HS256-signed JWT with `principals=["role:member"]`
and boots a server with `WSS_MUX_HANDSHAKE_SIGNING_KEY` set to a
per-run secret. The manifest at `tests/e2e/manifest.yaml` defines
three streams:

| Stream          | subscribe        | publish          | Purpose                                  |
|-----------------|------------------|------------------|------------------------------------------|
| `chat_messages` | `role:member`    | `role:member`    | Main happy-path roundtrip                |
| `presence`      | `*`              | `*`              | Wildcard audience smoke                  |
| `readonly`      | `role:member`    | (absent)         | Default-deny — `unauthorized_publish`    |

Env vars consumed by the SDK suites (set by the harness):

- `WSS_MUX_E2E_URL`        — `ws://127.0.0.1:<port>/stream`
- `WSS_MUX_E2E_TOKEN`      — JWT for `role:member`
- `WSS_MUX_E2E_PUSH_URL`   — `http://127.0.0.1:<port>/events`
- `WSS_MUX_E2E_PUSH_TOKEN` — bearer matching `WSS_MUX_PUSH_AUTH_TOKEN`
- `WSS_MUX_E2E=1`          — Node test gate for the TS suite

## CI

`.github/workflows/sdk-e2e.yml` runs this harness on PRs that touch
wire schema (`src/envelope.rs`, `src/manifest.rs`),
server-side dispatch (`src/server/**`), auth (`src/auth.rs`), either
SDK (`clients/**`), the harness itself (`tests/e2e/**`), or the
workflow. Other PRs skip the job — the path-gate keeps per-PR CI
fast while pinning wire-compat regressions whenever the surface
moves.

## Requirements

- `cargo` (Rust stable)
- `node` ≥ 20, `npm`
- `openssl`, `curl`, `bash`

No `jq` or `python` dependency — `mint_token.sh` builds the JWT with
just `openssl` so the harness runs on a vanilla GitHub Actions runner
and most laptop setups without extra installs.
