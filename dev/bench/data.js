window.BENCHMARK_DATA = {
  "lastUpdate": 1779410510082,
  "repoUrl": "https://github.com/alternet-dev/wss-mux",
  "entries": {
    "wss-mux benchmarks": [
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
          "id": "a1c638e94507584c16ead668312d7d5c2f07b6f8",
          "message": "ci: publish + track benchmark results (perf workflow) (#44)\n\n* ci: publish + track benchmark results (perf workflow)\n\n* ci: move perf continue-on-error to step level so the check stays green",
          "timestamp": "2026-05-21T01:25:02-06:00",
          "tree_id": "08750aa7d26fb57384fc9591e52257b01a3a09ae",
          "url": "https://github.com/alternet-dev/wss-mux/commit/a1c638e94507584c16ead668312d7d5c2f07b6f8"
        },
        "date": 1779348737313,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 255,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1637,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3710,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29679,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 327,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2386,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29354,
            "range": "± 1364",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 372025,
            "range": "± 21412",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3503821,
            "range": "± 212204",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 140,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 132,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31756,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 156,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 73,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 314,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 99,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3921,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 148,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 42704,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 668,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 373128,
            "range": "± 11929",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7331,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 130,
            "range": "± 0",
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
          "id": "000e2741684b85c683d5627802045ccc59591bc5",
          "message": "feat: add in-repo load and stress harness binary (#45)\n\nAdds `wss-mux-loadgen`, a black-box harness measuring throughput,\nlatency, and connection-count behavior against either an in-process\nwss-mux (spawned on an ephemeral port — self-contained) or an external\ninstance via `--target`. Three scenarios — `throughput`, `latency`,\n`connections` — each printing a human report or, with `--json`, a\nsingle parseable object that stamps its mode.\n\nNo new dependencies: the harness reuses crates already in\n`[dependencies]`, and pushes go out via `.body()` (not reqwest's\n`.json()`) so the production feature set is unchanged. It is a separate\n`[[bin]]` — the shipped `wss-mux` binary is untouched — and its only\ntests are pure-function unit tests, keeping load-test flakiness out of\nthe gate.",
          "timestamp": "2026-05-21T10:56:44-06:00",
          "tree_id": "2309a436afec86bf89815ce6b89b2f02aa888c0e",
          "url": "https://github.com/alternet-dev/wss-mux/commit/000e2741684b85c683d5627802045ccc59591bc5"
        },
        "date": 1779383015931,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 301,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1912,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 4018,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 34141,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 332,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2360,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28626,
            "range": "± 2182",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 364105,
            "range": "± 23223",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4594391,
            "range": "± 321657",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 169,
            "range": "± 469",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 154,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33663,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 180,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 77,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 79,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 338,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2263,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 149,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 38392,
            "range": "± 814",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 378611,
            "range": "± 2248",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6374,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 137,
            "range": "± 1",
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
          "id": "99260c69330d0ff7fa86a9b1f24ac76bb62011ea",
          "message": "feat: multi-instance peer-fleet load testing in the harness (#46)\n\n* feat: multi-instance peer-fleet load testing in the harness\n\nAdds a `--peers N` flag to `wss-mux-loadgen` that spins up an in-process\nfleet of N+1 wss-mux instances, each configured with the others as\npeers. With a fleet, producers push to the ingress instance and\nsubscribers are held on another, so the measured path crosses a\npeer-relay hop.\n\nThe throughput and latency reports gain a relay section: events\nrelayed, per-peer sends, and the `relay_events_unwanted ÷\nrelay_events_relayed` waste ratio, scraped and summed fleet-wide. That\nratio is the signal the deferred selective-relay work is data-gated on\n— it measures 2.0 for a 4-node fleet with subscribers on one instance,\n0.0 for a single-peer pair. The throughput drop field is narrowed to\nthe `overflow` reason, since benign `no_subscribers` drops dominate on\na non-subscriber fleet instance and are already captured by the relay\nwaste metric. No new dependencies; `--peers` is in-process-only.\n\n* feat: named broadcast/concentrated/keyed topologies for fleet runs\n\nAdds a `--topology` flag wrapping subscriber-placement and key-\ncardinality presets, so each fleet run models a real situation and the\nrelay-waste numbers are comparable across runs:\n\n- `broadcast` — one unkeyed stream, subscribers balanced across the\n  fleet (a popular feed behind a healthy load balancer)\n- `concentrated` — one unkeyed stream, all subscribers on one instance\n  (a misrouting load balancer)\n- `keyed` — many small-audience keys, subscribers scattered (chat rooms\n  or per-user channels)\n\nThe throughput and latency reports stamp the topology. Keyed\nsubscribers are scattered with an inline SplitMix64 hash so a key's\naudience lands on instances uncorrelated with the key — no new\ndependency. The waste ratios on a 4-node fleet measure 0.00 / 2.00 /\n1.00 for broadcast / concentrated / keyed, climbing to 0.02 / 6.00 /\n4.23 on an 8-node fleet.",
          "timestamp": "2026-05-21T12:30:15-06:00",
          "tree_id": "c7a6ec68a4416a3b0c7b8adbe58730e83ee84e3c",
          "url": "https://github.com/alternet-dev/wss-mux/commit/99260c69330d0ff7fa86a9b1f24ac76bb62011ea"
        },
        "date": 1779388622330,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 309,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1879,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3887,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33582,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 332,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2394,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28813,
            "range": "± 1932",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 364407,
            "range": "± 26035",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4910263,
            "range": "± 306160",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 160,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 139,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33368,
            "range": "± 554",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 167,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 77,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 80,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 352,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3512,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 150,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 38481,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 740,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 378544,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6327,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 137,
            "range": "± 5",
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
          "id": "864d40880b77f66118ca7aed6f03acd4519db58f",
          "message": "feat: add traffic-oddity stress scenarios to the load harness (#47)\n\n* feat: add traffic-oddity stress scenarios to the load harness\n\nAdds six `wss-mux-loadgen` subcommands that each drive a pathological\ncondition and report an expected-vs-observed \"degraded as designed\"\nverdict — never a gating assert:\n\n- slow-consumer — a subscriber stops reading on a small-queue stream\n- dead-peer — relay to a guaranteed-closed peer\n- coalesce-saturate — burst past a small relay-coalescing queue\n- reconnect-storm — open, drop, and re-establish N connections in waves\n- payload-cap — POST an oversized payload to a capped stream\n- rate-limit — flood inbound frames past a low rate limit\n\nEach provisions its own in-process instance via a new `ServerProfile`\n(rate-limit / coalescing / dead-peer variants), scrapes `/metrics`, and\nconfirms wss-mux degrades the way the docs describe. The harness's\nmanifest grows `loadgen_slow` and `loadgen_capped` streams. The\nreconnect storm is bounded so it cannot exhaust the OS ephemeral-port\nspace. No new dependencies; the only tests are pure-function units.\n\n* feat: stagger reconnect-storm connects with a jitter window\n\nAdds `--jitter-ms` (default 250) to the reconnect-storm scenario —\neach wave's connects are spread across the window instead of firing\nin a single instant; `--jitter-ms 0` recovers the pure thundering herd.\n\nWithout jitter a 10k-connection wave overflows the TCP accept backlog\nand loses ~4.5% of connects to SYN drops; staggered over 250ms the\nsame 10k connects 100%. The default count=100 run is unaffected.",
          "timestamp": "2026-05-21T16:28:48-06:00",
          "tree_id": "8eb340a82abd30d6060f5c76f5b63c45aa00b6fe",
          "url": "https://github.com/alternet-dev/wss-mux/commit/864d40880b77f66118ca7aed6f03acd4519db58f"
        },
        "date": 1779402937121,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 255,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1649,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3768,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29342,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 330,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2367,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29300,
            "range": "± 1580",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 404379,
            "range": "± 36656",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5846286,
            "range": "± 472659",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 140,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 137,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31984,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 163,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 72,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 75,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 355,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 100,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2279,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 148,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 37442,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 667,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 374505,
            "range": "± 4161",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7447,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 132,
            "range": "± 1",
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
          "id": "57448776c1fd008d486d4920c518184c9d61c326",
          "message": "docs: add traffic-oddities consumer runbook (#48)\n\nAdds a \"Traffic oddities — a consumer runbook\" section to\ndocs/operations.md, written from what the load harness revealed: one\nentry per anomaly (slow consumer, oversized payload, inbound flood,\nreconnect storm, dead peer, saturated relay-coalescing queue) — each\nas what you observe, what wss-mux is doing, and the embedder-side fix,\nciting the exact metric and wire code and reproducible with\n`wss-mux-loadgen`.\n\nAlso adds a cross-reference from docs/embedding.md and a `cargo bench`\n+ harness note to AGENTS.md, and corrects two stale lines in passing:\noperations.md cited a relay-flush restart metric removed in #40, and\nAGENTS.md still said overflow closes the connection (keep-open\nper-subscription since v0.4).",
          "timestamp": "2026-05-21T17:41:53-06:00",
          "tree_id": "cd5c7a91d6d7167029317509c218a4ac7de6b4f9",
          "url": "https://github.com/alternet-dev/wss-mux/commit/57448776c1fd008d486d4920c518184c9d61c326"
        },
        "date": 1779407306219,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 300,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1904,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 4278,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 35467,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 333,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2409,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28820,
            "range": "± 1913",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 386380,
            "range": "± 30054",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3740197,
            "range": "± 260952",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 160,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 140,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33507,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 166,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 366,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 103,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3803,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 148,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 21302,
            "range": "± 535",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 740,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 378373,
            "range": "± 5188",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6697,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 136,
            "range": "± 0",
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
          "id": "d7b5dbc9791cbd33be839bb014bb6b8268049e85",
          "message": "docs: align embedding.md multi-instance guidance with peer-relay (#49)\n\nThe \"Multi-instance\" and \"Producer-pod count\" subsections of\ndocs/embedding.md predated peer-relay — they said running a fleet\nrequires the producer to broadcast every event to every instance and\nreferenced a planned v0.3 Redis pubsub adapter. Both are wrong as of\nv0.5: wss-mux ships peer-relay (a producer pushes each event once to\nany single instance, which relays to its peers), and the Redis adapter\nwas rejected.\n\nRewrites \"Multi-instance\" to describe peer-relay and cross-reference\ndocs/operations.md, and drops the obsolete \"Producer-pod count\"\nsubsection — its M×N producer-amplification premise no longer exists.",
          "timestamp": "2026-05-21T17:49:14-06:00",
          "tree_id": "31725dc26bd1a22fc0dddc67760c76070a0daf1b",
          "url": "https://github.com/alternet-dev/wss-mux/commit/d7b5dbc9791cbd33be839bb014bb6b8268049e85"
        },
        "date": 1779407758789,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 254,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1643,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3670,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29866,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 335,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2468,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29936,
            "range": "± 1812",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 335458,
            "range": "± 21220",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4580005,
            "range": "± 275622",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 137,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 138,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31960,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 162,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 72,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 337,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2957,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 149,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 21092,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 662,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 401954,
            "range": "± 26802",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7292,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 132,
            "range": "± 0",
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
          "id": "8c0e9890240591e1456211958e97116fceae29ec",
          "message": "docs: remove roadmap.md; track forward-looking work in GitHub issues (#53)\n\nRepo docs describe shipped state; a roadmap is forward-looking and\ndrifts from reality. Remove docs/roadmap.md and relocate its three\nforward-looking items to GitHub issues: v1.0 stable contracts (#50),\ninterest-based selective relay (#51), opt-in inbound overload signal\n(#52).\n\nClean up the dangling references: reading-order entries in README and\nAGENTS.md, two prose mentions in operations.md, and two http.rs doc\ncomments. Per-version history stays in git tags.",
          "timestamp": "2026-05-21T18:35:08-06:00",
          "tree_id": "c4f0d5746475543b1c6c46c21c70952cb664ffe8",
          "url": "https://github.com/alternet-dev/wss-mux/commit/8c0e9890240591e1456211958e97116fceae29ec"
        },
        "date": 1779410509753,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 319,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1901,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3944,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33394,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 336,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2385,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28425,
            "range": "± 1533",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 396639,
            "range": "± 23391",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3603357,
            "range": "± 234165",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 158,
            "range": "± 466",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 139,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33601,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 169,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 314,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3712,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 149,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 38394,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 381483,
            "range": "± 1917",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6311,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 136,
            "range": "± 4",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}