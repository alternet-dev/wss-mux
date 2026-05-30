window.BENCHMARK_DATA = {
  "lastUpdate": 1780127507301,
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
          "id": "8010420f3c8a763326322e231f017b9fef777ec3",
          "message": "ci(release): add cargo-publish job for wss-mux-client (#98)\n\nMirrors the npm-publish job's structure so the v0.6.0 tag ships the\nRust SDK to crates.io alongside @alternet/wss-mux-client to npm and\nthe binaries / Homebrew formula to their existing channels.\n\n## Posture\n\nIdempotent: queries `https://crates.io/api/v1/crates/wss-mux-client/$VERSION`\nbefore publishing. 200 ⇒ already on the registry, skip cleanly. 404 ⇒\nnew version (or first-ever publish if the crate exists), proceed.\nSame retag-friendly behavior as the npm-publish + github-release\njobs — a partial release can be re-driven without manually skipping\nalready-completed steps.\n\n## Validation\n\nTwo pre-flight checks:\n\n1. `CARGO_REGISTRY_TOKEN` secret must be set. If missing, fail fast\n   with the link to crates.io's token page rather than letting\n   `cargo publish` produce a less-actionable \"unauthorized\" error.\n2. `Cargo.toml` version must match the git tag (stripped of the `v`\n   prefix). Catches a bumped tag with a stale `Cargo.toml`, which\n   would otherwise publish the wrong version.\n\nBoth checks fail the workflow before any registry call.\n\n## Auth\n\nToken-based — crates.io's Trusted Publishing is not GA as of\nmid-2026, so the OIDC pattern used by `npm-publish` doesn't apply\nhere. `CARGO_REGISTRY_TOKEN` is a publish-scoped API token from\ncrates.io (Account → Settings → New Token), stored as a repo\nsecret. The workflow injects it only into the publish step; no\nother step can read it.\n\n## Maintainer action items before tagging v0.6.0\n\n- `CARGO_REGISTRY_TOKEN` repo secret created (one-time).\n- `wss-mux-client` name claimed on crates.io (one-time `cargo\n  publish` of the current version, or an org claim).\n\nBoth are no-ops for subsequent releases.",
          "timestamp": "2026-05-29T22:42:31-06:00",
          "tree_id": "966b009f3bb1b9d6bc4f7b80979ce6bafae90310",
          "url": "https://github.com/alternet-dev/wss-mux/commit/8010420f3c8a763326322e231f017b9fef777ec3"
        },
        "date": 1780116205080,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1097042,
            "range": "± 53480",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 10903136,
            "range": "± 361779",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 109655414,
            "range": "± 2242981",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 43485,
            "range": "± 1400",
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
          "id": "ccb5f427f5f5ca007f1b65340a1cad2a5c825c47",
          "message": "docs: v0.6 sweep — WS publish, SSE read, SDKs, presence_svc pattern (#100)\n\nBrings the three repo-level docs in line with the v0.6 wire surface\nthat landed across #87–#94 and #95.\n\n## architecture.md\n\n- Dispatch diagram now shows the SSE read endpoint (HTTP server) and\n  publish on the WS server, matching `build_app`'s route table.\n- New \"Unified dispatch\" callout: HTTP push, WS publish, WS subscribe,\n  and SSE read all share the dispatcher and registry — the four wire\n  surfaces differ only in framing.\n- HTTP server section gains a paragraph on `GET /events/:stream`\n  (auth matrix + dispatcher-side parity with WS subscribe).\n- WS server section now covers the `publish` frame: audience check,\n  per-stream payload cap, per-source rate limit, dispatch via the\n  same path as HTTP `POST /events`. Notes that publish rejections\n  are keep-open (same posture as `overflow`).\n\n## embedding.md\n\n- New \"Client SDKs\" section pointing to the TS and Rust SDKs under\n  `clients/` and noting they're wire-compatible.\n- New \"Building a presence service on top\" section — the canonical\n  end-to-end example the v0.6 design was shaped around. Shows the\n  manifest stanza, a browser-side heartbeat using the TS SDK, and an\n  HTTP SSE consumer pattern for a backend presence_svc. Closes with\n  the schematic of clients ↔ wss-mux ↔ presence_svc.\n\n## README.md\n\n- Quick start's Subscribe example switches from a raw WebSocket\n  snippet to the TS SDK quickstart — more representative of how\n  consumers actually integrate. Same example also demonstrates\n  `client.publish()` over the same connection.\n- Adds pointers to the Rust SDK and the SSE read endpoint for the\n  non-WS consumer story.\n\nNo changes to `docs/protocol.md` — wire surface for publish frames\nand `unauthorized_publish` / `publish_payload_too_large` was already\ndocumented when the frames landed.",
          "timestamp": "2026-05-29T22:58:04-06:00",
          "tree_id": "73f0fc3ad7213ae0210fc9cf6f278c6788b84fb2",
          "url": "https://github.com/alternet-dev/wss-mux/commit/ccb5f427f5f5ca007f1b65340a1cad2a5c825c47"
        },
        "date": 1780117139817,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1107369,
            "range": "± 61055",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11041599,
            "range": "± 388174",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 109940537,
            "range": "± 2024611",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 44057,
            "range": "± 2192",
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
          "id": "db7eada2577339c8f58d21ed6539cdceabb39738",
          "message": "chore: bump version to 0.6.0 (#101)\n\nCuts v0.6.0, the WS-publish + SDK release. Three crates / packages\nbump in lockstep:\n\n- `wss-mux` (server) `0.5.3` → `0.6.0`\n- `wss-mux-client` (Rust SDK) `0.5.3` → `0.6.0` — first crates.io publish\n- `@alternet/wss-mux-client` (TS SDK) `0.5.3` → `0.6.0`\n\n## What ships in v0.6.0\n\nWire surface (server):\n\n- `publish` ClientFrame variant (#87, #88) — a WS client emits an\n  event over the connection it already holds open. Same downstream\n  dispatch as HTTP `POST /events`.\n- Per-stream `publish` audience in the manifest (#87) — default-deny,\n  parallel to the existing `subscribe` field. Field rename from\n  `audience` → `subscribe` is the wire-breaking part (pre-1.0\n  manifest schema bump).\n- `unauthorized_publish` + `publish_payload_too_large` error codes\n  (#88), keep-open.\n- `PerSourceRateLimiter` keyed by JWT `sub` (#89, #90) — per-source\n  publish budget, sweeper task, configurable via\n  `WSS_MUX_WS_PUBLISH_RATE` / `_BURST` / `_REQUIRE_SUB` /\n  `_IDLE_TTL_SECS`.\n- `GET /events/:stream` SSE read endpoint (#91) — the backend-\n  consumer story; auth supports shared-bearer\n  (`WSS_MUX_READ_AUTH_TOKEN`) and JWT.\n\nClient SDKs:\n\n- TypeScript SDK gains `client.publish()` (#92), `publishSettleMs`\n  builder option, `unauthorized_publish` / `publish_payload_too_large`\n  in the `ErrorCode` union.\n- Rust SDK (`clients/rust/`) lands as a new crate (#93, #94) —\n  subscribe + publish, reconnect + token-refresh + heartbeat\n  internal, typed errors. `connect_async`-based, minimal deps.\n- Cross-SDK e2e harness (#96) — single-shell entrypoint that boots\n  the server, mints a JWT, runs both SDKs' gated e2e suites; new\n  path-gated `sdk-e2e.yml` workflow.\n- Rust SDK criterion benches against an in-process server (#97):\n  publish_self_roundtrip, publish_throughput, subscribe_one.\n\nCI / release:\n\n- `cargo-publish` job in `release.yml` (#98) — idempotent via\n  crates.io HEAD check, validates Cargo.toml vs tag, uses\n  `CARGO_REGISTRY_TOKEN`. TP migration tracked separately in #99.\n\nDocs:\n\n- `docs/architecture.md` + `docs/embedding.md` swept for the new\n  surface (#100), including the presence_svc pattern.\n\n## Maintainer action items before the tag fires\n\n- `CARGO_REGISTRY_TOKEN` secret set (token scoped to\n  `wss-mux-client`, `publish-new` + `publish-update`).\n- crates.io has space for the `wss-mux-client` name (first publish\n  will claim it).",
          "timestamp": "2026-05-29T22:58:09-06:00",
          "tree_id": "957add102e32fd1ebea536699cceee102c0e7277",
          "url": "https://github.com/alternet-dev/wss-mux/commit/db7eada2577339c8f58d21ed6539cdceabb39738"
        },
        "date": 1780117144194,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1163666,
            "range": "± 34867",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11408863,
            "range": "± 190045",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 113162331,
            "range": "± 1555627",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 35072,
            "range": "± 2228",
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
          "id": "46374939844e2750bcb0eeddab6dfc6797f2585c",
          "message": "fix(oidc): split discovery_url from issuer (closes #86) (#104)\n\nLets the public `iss` value tokens carry diverge from the URL\nwss-mux fetches discovery from. The Keycloak\n`KC_HOSTNAME_BACKCHANNEL_DYNAMIC` pattern (and analogous setups for\nother IdPs) deliberately splits these — tokens minted on either the\npublic or in-cluster host claim the public issuer, while metadata\nendpoints serve via whichever host the request actually arrived on.\n\n## Before\n\n`OidcConfig.issuer` did two jobs at once: token-iss validation\n(`set_issuer(&[issuer])` in auth) AND the discovery base\n(`<issuer>/.well-known/openid-configuration` in oidc). Operators\nrunning Keycloak BACKCHANNEL_DYNAMIC had to point\n`WSS_MUX_OIDC_ISSUER` at the public URL (token-iss matches) and then\npin `WSS_MUX_OIDC_JWKS_URL` to the in-cluster endpoint to skip\ndiscovery entirely — sidestepping OIDC's \"the IdP tells you the\nendpoints\" guarantee.\n\n## After\n\nNew optional `OidcConfig.discovery_url` (env\n`WSS_MUX_OIDC_DISCOVERY_URL`). When set, it's the discovery base;\nwhen unset, the discovery base is `issuer` (existing behavior).\nToken-iss validation is unchanged — still pinned to `issuer`.\n`jwks_url` is also unchanged; if set it still short-circuits the\nwhole discovery dance.\n\n```bash\nWSS_MUX_OIDC_ISSUER=https://auth.example.com/realms/x       # token iss\nWSS_MUX_OIDC_DISCOVERY_URL=http://keycloak:8080/realms/x    # discovery base\n# jwks_url left unset — the discovery doc tells us where keys live,\n# and BACKCHANNEL_DYNAMIC makes that URL in-cluster too.\n```\n\n## Tests\n\n- `src/oidc.rs`: `discovery_url_base_overrides_issuer` —\n  resolve_jwks_url fetches discovery at the in-cluster URL and\n  returns the in-cluster jwks_uri the doc carries.\n- `src/oidc.rs`: `jwks_url_still_short_circuits_discovery_url` —\n  explicit jwks_url still wins over discovery_url.\n- `src/config.rs`: `oidc_discovery_url_is_read_independently_of_issuer`\n  — env-var plumbing.\n\nPlus the existing OIDC test fixtures + the integration test's\n`oidc_cfg` helper threaded for the new field.\n\n## Out of scope\n\n- Caching changes (discovery cache is already separate from JWKS).\n- Validation strictness (`set_issuer(&[issuer])` is correct as-is).\n- Multi-issuer (still one IdP per multiplexer).\n\n## Docs\n\n- `docs/embedding.md` OIDC section: new env var + a paragraph on\n  the BACKCHANNEL_DYNAMIC use case.\n- `README.md` config table: new row, JWKS row clarified to reference\n  the discovery base.",
          "timestamp": "2026-05-30T00:42:44-06:00",
          "tree_id": "3a151ed0d74fda485428b16b2e04ce66a009d6cb",
          "url": "https://github.com/alternet-dev/wss-mux/commit/46374939844e2750bcb0eeddab6dfc6797f2585c"
        },
        "date": 1780123421452,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1107594,
            "range": "± 63773",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11318070,
            "range": "± 347056",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 113246380,
            "range": "± 3024940",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 44548,
            "range": "± 3853",
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
          "id": "a34f57c765fade212775697a53532f3741ca3510",
          "message": "feat(cli): wss-mux validate-manifest subcommand (#102) (#105)\n\nAdds `wss-mux validate-manifest <path>`: parses + validates a\nstreams manifest using upstream's own parser, then exits 0 with a\none-line summary or 1 with the validation error. No env vars\nrequired; no server runtime spun up.\n\n## Why\n\nEmbedders that codegen the streams manifest (e.g. application-\ntemplate's Python AST walker) need to drift-check the generated YAML\nagainst upstream's schema. Today they re-implement enough of\n`src/manifest.rs` to read the file, which means the v0.6 `audience`\n→ `subscribe` rename + new `publish` field broke them silently — and\nevery future schema evolution carries the same hazard.\n\nThis subcommand makes upstream's parser the source of truth: the\nembedder's CI pre-flight shells out instead of carrying its own copy\nof the schema.\n\n## Surface\n\n```\n$ wss-mux validate-manifest config/wss-mux/streams.yaml\nok: 2 streams loaded (notification_banner, suspension_status_card)\n$ echo $?\n0\n\n$ wss-mux validate-manifest broken.yaml\nerror: ...\n$ echo $?\n1\n\n$ wss-mux validate-manifest\nusage: wss-mux validate-manifest <path>\n$ echo $?\n2\n```\n\n## Dispatch\n\n`main()` now inspects `argv[1]` synchronously before any tokio\nruntime spin-up:\n\n- `validate-manifest` → run synchronously, exit.\n- `help` / `--help` / `-h` → print usage, exit 0.\n- anything else (or no arg) → fall through to `server_main()`, which\n  carries the existing async server behavior verbatim. The new\n  surface is **additive** — `wss-mux` with no args still starts the\n  server unchanged.\n\nThe dispatch lives ahead of `Config::from_env()`, so embedders don't\nneed to satisfy the server's env-var contract when they're just\nvalidating a file.\n\n## Tests\n\n`tests/integration/validate_manifest.rs` spawns the built binary via\n`env!(\"CARGO_BIN_EXE_wss-mux\")` and asserts:\n\n- valid YAML → exit 0, stdout includes the stream count + names\n- invalid YAML → non-zero exit, stderr mentions \"error\"\n- missing path → non-zero exit\n- missing positional arg → non-zero exit with a usage hint\n\nTiny in-test tempdir helper rather than a new dev-dep.\n\n## Out of scope\n\n- `--json` structured output. Marked \"maybe\" in #102; if an embedder\n  needs structured diagnostics we can add it as a follow-up. Today's\n  callers grep stdout / treat exit code as the signal.\n- Wire-protocol validation, audience-set validation against an\n  external IdP, etc. — separate concerns per the issue.\n\n## Docs\n\n`README.md` gains a one-paragraph note under the operational\nsection.",
          "timestamp": "2026-05-30T01:32:04-06:00",
          "tree_id": "a2bdd2bdc8955cbc1a90c4dbc2f49dd3c54c4dd8",
          "url": "https://github.com/alternet-dev/wss-mux/commit/a34f57c765fade212775697a53532f3741ca3510"
        },
        "date": 1780126378539,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1111181,
            "range": "± 42647",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11305260,
            "range": "± 230956",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 113188562,
            "range": "± 2141585",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 34755,
            "range": "± 5929",
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
          "id": "ebd4de160f381268afd7ed16933812812c5fc1af",
          "message": "docs(client-ts): pin getToken contract; assert fresh token on wire after 4401 (closes #103) (#106)\n\nTwo pieces:\n\n## Doc the contract\n\nThe TS SDK's `getToken` callback has a precise contract the README\nonly gestured at: called on initial connect AND on close-code 4401,\n*and the freshly returned value reaches the wire* (no stale-cache\nwindow across a 4401-driven reconnect). Other reconnects reuse the\ncached token so a flapping connection doesn't hammer the issuer.\n\nEmbedders rely on this for token-rotation safety nets — a SPA's\nshort-lived JWT can refresh out-of-band, and the SDK auto-recovers\non the next 4401 without re-mounting the client.\n\nREADME's getToken section now spells out:\n\n- Exactly when the SDK invokes it (initial connect + 4401 reconnect).\n- That the fresh value reaches the wire on the post-4401 connection,\n  not a cached previous value.\n- The non-4401 reconnect caching behavior, so consumers don't expect\n  refresh on every disconnect.\n- Pre-emptive refresh is out of scope (app-layer concern).\n\n## Strengthen the test\n\n`reconnect.test.mjs` already had \"4401 close triggers fresh\ngetToken() call\", which asserts *getToken was called twice* and the\ntwo returned tokens differ — but not that the second one actually\nmakes it onto the wire. That's a real failure mode (cache-\ninvalidation order bug, stale closure capture, etc.) and not\nsomething the existing test would catch.\n\nNew test `4401 reconnect sends the freshly-fetched token on the\nwire`: the mock server records auth frames per connection; the\ntest asserts the post-4401 connection's auth-frame `token` is\nexactly the new value returned by getToken, not the initial one.\n\nSuite total: 20 pass / 4 skip (e2e), was 19 / 4. typecheck clean.\n\n## Out of scope\n\n- Pre-emptive refresh strategy. Per the issue, that's an app-layer\n  feature — the SDK only handles the 4401-driven safety net.\n- Refresh contracts for the Rust SDK. Same shape but different\n  language idioms; a follow-up if asked.",
          "timestamp": "2026-05-30T01:37:03-06:00",
          "tree_id": "64eb35311c85b49c4f029b1211a6f1e4886a727d",
          "url": "https://github.com/alternet-dev/wss-mux/commit/ebd4de160f381268afd7ed16933812812c5fc1af"
        },
        "date": 1780126681354,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1111004,
            "range": "± 44351",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11173928,
            "range": "± 262101",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 112339878,
            "range": "± 1745850",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 29017,
            "range": "± 1389",
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
          "id": "4211a5be187d4c41110e4f1b186bb12a7a9d645c",
          "message": "chore: bump version to 0.6.1 (#107)\n\nPatch release on top of 0.6.0. Three packages move in lockstep:\n\n- `wss-mux` (server) 0.6.0 → 0.6.1\n- `wss-mux-client` (Rust SDK) 0.6.0 → 0.6.1\n- `@alternet/wss-mux-client` (TS SDK) 0.6.0 → 0.6.1\n\n## What ships\n\n- OIDC: split discovery_url from issuer (#104) — new\n  `WSS_MUX_OIDC_DISCOVERY_URL` env var lets the public `iss` value\n  tokens carry diverge from the URL wss-mux fetches discovery from.\n  Resolves the Keycloak `KC_HOSTNAME_BACKCHANNEL_DYNAMIC` pain point\n  where the only workaround was pinning `WSS_MUX_OIDC_JWKS_URL` and\n  giving up the rest of the discovery doc.\n- `wss-mux validate-manifest <path>` subcommand (#105) — parses +\n  validates a streams manifest using upstream's own parser. Exits 0\n  with a one-line summary, 1 on validation error, 2 on usage error.\n  No env vars required, no server runtime. For embedders' CI\n  pre-flight on codegen'd manifests.\n- TS SDK: getToken contract pinned in the README + a wire-level test\n  asserting the freshly-fetched token actually reaches the auth\n  frame on a 4401-driven reconnect (#106). No behavior change in\n  the SDK; characterization test locks the invariant against\n  regression.\n\nThe Rust SDK has no source changes; the version bump is the\nlockstep half. crates.io publishes a 0.6.1 with identical code to\n0.6.0 so the trio's versions stay aligned.\n\n## No wire-breaking changes\n\nPatch release — all wire surfaces (envelope, manifest schema, error\ncodes, close codes, subprotocol) are byte-identical to 0.6.0. A\n0.6.0 client speaks to a 0.6.1 server and vice versa, no changes.",
          "timestamp": "2026-05-30T01:50:52-06:00",
          "tree_id": "dc31810eff8881947a81cf1fdbe87e0b364401c6",
          "url": "https://github.com/alternet-dev/wss-mux/commit/4211a5be187d4c41110e4f1b186bb12a7a9d645c"
        },
        "date": 1780127507010,
        "tool": "cargo",
        "benches": [
          {
            "name": "publish_self_roundtrip",
            "value": 1097136,
            "range": "± 56290",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/10",
            "value": 11049632,
            "range": "± 197432",
            "unit": "ns/iter"
          },
          {
            "name": "publish_throughput/100",
            "value": 111445562,
            "range": "± 2125180",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe_one",
            "value": 42818,
            "range": "± 1492",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}