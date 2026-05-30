window.BENCHMARK_DATA = {
  "lastUpdate": 1780115407752,
  "repoUrl": "https://github.com/alternet-dev/wss-mux",
  "entries": {
    "wss-mux-client SDK benchmarks": [
      {
        "commit": {
          "author": {
            "email": "167108037+evan-macgregor@users.noreply.github.com",
            "name": "Evan MacGregor",
            "username": "evan-macgregor"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "270dc120d132b2fd712ecf279a09d8768513be44",
          "message": "feat(client-rust): SDK e2e benches against in-process wss-mux (#97)\n\nAdds `cargo bench --bench sdk_e2e` in the Rust client crate. Each\nbench spawns a real `wss-mux` server in-process on an ephemeral port\nand drives the SDK against it, so the numbers reflect the full SDK ↔\nserver roundtrip (serialize → WS write → server parse + audience +\ndispatch → WS read → SDK deserialize → channel hand-off) rather than\na mock or microbench.\n\n## Benches\n\n- `publish_self_roundtrip` — single publish + own subscribe recv;\n  ~1.2 ms on M-series loopback.\n- `publish_throughput/{10,100}` — N publishes back-to-back, then\n  receive all N. Reports events/sec via criterion's `Throughput`.\n- `subscribe_one` — bind a subscription, drop it (sends unsubscribe);\n  ~20 µs on loopback.\n\n## Dependencies\n\nDev-only — published SDK dependency graph is unaffected:\n\n- `wss-mux` (path = \"../..\") — server crate, for the in-process bind.\n- `axum` — for `axum::serve` in the bench harness.\n- `jsonwebtoken` — HS256-sign the bench's JWT directly.\n- `criterion` with `async_tokio` — bench driver.\n\n## CI\n\n`.github/workflows/perf.yml` gains a `sdk-benchmarks (report-only)`\njob mirroring the existing wss-mux microbench job. Same posture: never\nfail the build, comment alerts on PRs, persist the trend on trunk to\nthe gh-pages dashboard under `dev/bench/sdk/`. `--measurement-time 2\n--warm-up-time 1` keeps the whole job well under a minute on a hosted\nrunner.\n\n## Settle window\n\nThe client builder is configured with `publish_settle: Duration::ZERO`\nso the publish() future resolves immediately and the bench's `recv()`\ncaptures the actual server roundtrip latency. The throughput bench\nexploits this same property to publish N back-to-back before draining\nthe receiver.\n\n## Rate limiter\n\nThe bench-side `Config` zeroes `inbound_rate_per_sec`,\n`inbound_burst`, and the `ws_publish_*` knobs. The bench's whole\npurpose is to measure throughput from one principal hammering one\nstream — the limiter behavior is its own bench (`per_source_rate_limit`\nin the server crate).",
          "timestamp": "2026-05-29T19:36:16-06:00",
          "tree_id": "0385db0507740f7fe4e51f3fa5a58715c9e78335",
          "url": "https://github.com/alternet-dev/wss-mux/commit/270dc120d132b2fd712ecf279a09d8768513be44"
        },
        "date": 1780105115102,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1102074,
            "range": "± 62982",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11047408,
            "range": "± 268715",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 110412440,
            "range": "± 2607964",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 44251,
            "range": "± 1567",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "167108037+evan-macgregor@users.noreply.github.com",
            "name": "Evan MacGregor",
            "username": "evan-macgregor"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ec62eb580268d0f38dc4ade198b31596f683a0c4",
          "message": "feat(e2e): cross-SDK harness + Rust e2e + TS e2e expansion (#96)\n\nAdds a repo-level e2e harness that boots a real `wss-mux` binary,\nmints a JWT, and runs both SDK e2e suites against it. Cross-SDK wire\ncompatibility now has an end-to-end gate, not just per-SDK unit\ntests.\n\n## Harness\n\n`tests/e2e/`:\n\n- `manifest.yaml` — minimal three-stream manifest (`chat_messages`\n  for the happy path, `presence` for wildcard, `readonly` for\n  default-deny).\n- `mint_token.sh` — HS256-signed JWT minter using only openssl + bash\n  (no jq / python dependency).\n- `run.sh` — orchestration. Builds the release binary, picks a fixed\n  ephemeral port, boots the server with a per-run secret +\n  `WSS_MUX_HANDSHAKE_SIGNING_KEY` / `WSS_MUX_PUSH_AUTH_TOKEN` /\n  `WSS_MUX_READ_AUTH_TOKEN`, waits for `/health`, then runs each\n  SDK's gated e2e suite. Trap cleans up the server on any exit.\n- `README.md` — what's covered, env-var contract, requirements.\n\n```bash\ntests/e2e/run.sh         # rust + typescript\ntests/e2e/run.sh rust    # rust only\ntests/e2e/run.sh ts      # typescript only\n```\n\nLocal: ~5 s including server boot. CI: ~30 s on a cold runner.\n\n## Rust e2e (`clients/rust/tests/e2e.rs`)\n\n4 gated tests, skip if `WSS_MUX_E2E_URL` is unset so `cargo test` on\ntrunk doesn't try to dial a missing server:\n\n- connect + subscribe + WS publish own-event roundtrip\n- publish to readonly stream → `Protocol(unauthorized_publish)`\n- HTTP `POST /events` push observed by WS subscriber (cross-\n  transport)\n- unsubscribe stops event delivery\n\nThe HTTP-push helper hand-rolls a tiny tokio-based POST so the SDK\ncrate stays reqwest-free.\n\n## TS e2e (`clients/typescript/tests/e2e.test.mjs`)\n\nReplaces the single-subscribe scaffold with 4 gated tests\nsymmetric to the Rust suite, plus a cross-client publish→subscribe\ntest (one SDK instance publishes, another instance on the same\nserver observes).\n\n## CI (`.github/workflows/sdk-e2e.yml`)\n\nPath-gated to wire schema (`src/envelope.rs`, `src/manifest.rs`),\nserver dispatch (`src/server/**`), auth (`src/auth.rs`), either SDK\n(`clients/**`), the harness itself, or the workflow file. Other PRs\nskip the job — per-PR CI stays fast while wire-compat regressions\nare pinned whenever the surface moves.\n\n## Docs\n\n`tests/e2e/README.md` documents the harness end-to-end. Both SDK\nREADMEs gain a one-line pointer to it under Development.",
          "timestamp": "2026-05-29T22:29:11-06:00",
          "tree_id": "55a32026244353a1fba47cc5e6c20e56325325c1",
          "url": "https://github.com/alternet-dev/wss-mux/commit/ec62eb580268d0f38dc4ade198b31596f683a0c4"
        },
        "date": 1780115406395,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1114395,
            "range": "± 53050",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11127265,
            "range": "± 296871",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 110921524,
            "range": "± 1928042",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 41975,
            "range": "± 1356",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}