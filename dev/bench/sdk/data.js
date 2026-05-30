window.BENCHMARK_DATA = {
  "lastUpdate": 1780105115390,
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
      }
    ]
  }
}