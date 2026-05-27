window.BENCHMARK_DATA = {
  "lastUpdate": 1779863693000,
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
          "id": "025276849883a0fa51e831aef47df0429dcbe419",
          "message": "chore: bump version to 0.5.1 (#70)\n\nEstablishes the v0.5.1 release line. No behaviour change in this commit;\nthe v0.5.1 release ships the wire-subprotocol drop, the TypeScript SDK,\nand the multi-registry release fanout — all stacked on top of this PR.",
          "timestamp": "2026-05-26T19:43:43-06:00",
          "tree_id": "8af2ce73735ca178d82dc8165472b31336a8cb3f",
          "url": "https://github.com/alternet-dev/wss-mux/commit/025276849883a0fa51e831aef47df0429dcbe419"
        },
        "date": 1779846618841,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 253,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1674,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3651,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29597,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 337,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2412,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29570,
            "range": "± 1733",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 354594,
            "range": "± 38327",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3899628,
            "range": "± 173145",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 144,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31466,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 155,
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
            "range": "± 0",
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
            "value": 351,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2982,
            "range": "± 8",
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
            "value": 42816,
            "range": "± 509",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 660,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 372416,
            "range": "± 4595",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7043,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 133,
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
          "id": "05acc1904f65d38b74b2b50a27ce7c54a188eca2",
          "message": "feat(wire)!: drop version from WebSocket subprotocol identifier (#72)\n\nBREAKING CHANGE: the subprotocol Sec-WebSocket-Protocol identifier loses\nits version field. Clients previously offering `wss-mux.v1` or\n`wss-mux.v1.cbor` must now offer `wss-mux` or `wss-mux.cbor` to\nnegotiate successfully; the server returns HTTP 400 for any v0.5.0\nclient that still sends the old identifier.\n\nThis is a pre-1.0 wire break, allowed under the project's pre-1.0\nSemVer policy and documented in `docs/protocol.md`.\n\n## Why\n\nUnder the client/server lockstep model (see #54), the subprotocol's\nversion field was redundant: a client at a given release is paired with\na server at the same release by construction. Negotiation of *which*\nwire version to use is structural, not advertised. The `.v1` field\ncreated three different version concepts in play (server SemVer, SDK\nSemVer, wire-major) and exposed the inconsistency that wire-major didn't\nmove with server SemVer — even at minor/patch wire breaks pre-1.0.\n\nThe same argument that's driving #55 to drop URL versioning applies\nhere: under lockstep, version is implicit, so the identifier just\nidentifies the protocol.\n\n## Changes\n\n- `src/server/ws.rs`: `SUBPROTOCOL` is now `wss-mux`, `SUBPROTOCOL_CBOR`\n  is `wss-mux.cbor`. Negotiation logic, error message, and code comments\n  updated.\n- `src/bin/loadgen/client.rs`: sends `wss-mux` on upgrade.\n- `tests/integration/common.rs`: WebSocket connect helpers updated.\n- `docs/protocol.md`: §\"Subprotocol negotiation\" and §\"Forward\n  compatibility\" updated. The forward-compat section now explains why\n  the identifier doesn't carry a version under lockstep.\n- `README.md`: CBOR note updated.\n\n## Verification\n\n- `cargo build` — green.\n- `cargo test --all-targets` — all 262 tests pass.\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.",
          "timestamp": "2026-05-26T20:52:03-06:00",
          "tree_id": "574ad8d04524389ddd38fce4267c12dd8a9f9464",
          "url": "https://github.com/alternet-dev/wss-mux/commit/05acc1904f65d38b74b2b50a27ce7c54a188eca2"
        },
        "date": 1779850723597,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 256,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1649,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3743,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29730,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 333,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2391,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29371,
            "range": "± 1627",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 376259,
            "range": "± 25690",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4981490,
            "range": "± 378777",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 138,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 132,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31579,
            "range": "± 239",
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
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 72,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 352,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 99,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3818,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 147,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 37631,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 664,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 371408,
            "range": "± 4279",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7162,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 133,
            "range": "± 2",
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
          "id": "59bfd2d410da777f38abc4d6d3059795c4102a3f",
          "message": "feat(client): add @wss-mux/client TypeScript SDK (#54) (#68)\n\nBrowser TypeScript SDK wrapping the wss-mux v0.5 wire protocol, per #54.\nZero runtime dependencies; uses globalThis.WebSocket (constructor\ninjectable for non-browser hosts). Auto-reconnect with exponential\nbackoff; getToken() invoked on initial connect and on close code 4401\n(expired_token). Typed errors for the documented wss-mux error codes\nand close codes. Subscriptions are tracked locally and replayed on\nreconnect; the protocol's ack-by-absence semantics mean subscribe()\nresolves once the frame is sent, with server-side errors surfaced via\nopts.onError.\n\nNegotiates the `wss-mux` subprotocol (matching the version-less\nidentifier introduced in the wire-drop PR stacked just below). Does\nnot surface `v1` anywhere in the public API or examples; the URL is\noperator-controlled per the handshake response.\n\nTests run under node:test against a mock WebSocketServer (ws package);\none e2e test gated by WSS_MUX_E2E=1. CI workflow at\n.github/workflows/typescript-ci.yml runs typecheck + build + tests\non touches to clients/typescript/**.\n\nStacked on:\n- chore: bump version to 0.5.1\n- feat(wire)!: drop version from WebSocket subprotocol identifier\n\nPublishes to npm when the multi-registry release fanout PR also merges\nand v0.5.1 is tagged.",
          "timestamp": "2026-05-26T20:53:55-06:00",
          "tree_id": "5507c6780fd1e5b7b1130d9cc2caec33b5b73db8",
          "url": "https://github.com/alternet-dev/wss-mux/commit/59bfd2d410da777f38abc4d6d3059795c4102a3f"
        },
        "date": 1779850829762,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 302,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1886,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3928,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33668,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 330,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2472,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28511,
            "range": "± 1874",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 352451,
            "range": "± 26380",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3829246,
            "range": "± 228672",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 159,
            "range": "± 444",
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
            "value": 33653,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 166,
            "range": "± 7",
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 436,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 105,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 4385,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 151,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 41715,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 741,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 407477,
            "range": "± 2484",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6384,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 136,
            "range": "± 2",
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
          "id": "cda7967a076d6f845bd6f60fce37bab5a4f733ff",
          "message": "chore(client): rename to @alternet/wss-mux-client (#73)\n\nPublishes the SDK under the org-wide @alternet npm scope rather than\na project-specific @wss-mux scope. Mirrors the existing pattern of the\nshared Homebrew tap at alternet-dev/homebrew-tap — one org-level scope\nacross all alternet projects.\n\nThe @wss-mux scope is held in reserve; if wss-mux ever grows multiple\nnpm artifacts (e.g. a Node helper, a CLI, schema types), we can promote\nthis package to @wss-mux/client at that point. Migration would be a\ndeprecation notice on @alternet/wss-mux-client + a shim final-version\nthat re-exports from the new name. npm has no first-class redirects,\nso this is the standard pre-1.0 dance.\n\nChanges:\n- package.json `name` field.\n- package-lock.json regenerated via `npm install` (matching project\n  metadata only; no dependency change).\n- README install snippet and import in the quickstart.\n- examples/basic.ts import and header comment.\n\nVerification:\n- npm run typecheck — green.\n- npm run build — green.\n- npm test — 12 unit + reconnect tests pass; 1 e2e skipped.",
          "timestamp": "2026-05-26T23:16:35-06:00",
          "tree_id": "96fa5deb82d205700b7c5fbc0a1a9077bcdf6305",
          "url": "https://github.com/alternet-dev/wss-mux/commit/cda7967a076d6f845bd6f60fce37bab5a4f733ff"
        },
        "date": 1779859389928,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 297,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1900,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 4117,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33260,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 332,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2405,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28457,
            "range": "± 1189",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 367971,
            "range": "± 27850",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5311370,
            "range": "± 350812",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 158,
            "range": "± 6",
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
            "value": 33310,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 165,
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
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 363,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 4133,
            "range": "± 51",
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
            "value": 38802,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 374243,
            "range": "± 9206",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6552,
            "range": "± 19",
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
          "id": "e9f0f018b294c9d91b2ae0206b5a87a8fbc5d974",
          "message": "ci(release): multi-registry fanout on tag (npm + Homebrew + binaries) (#69)\n\n* ci(release): multi-registry fanout on tag (npm + Homebrew + binaries)\n\nExtends .github/workflows/release.yml so a v*.*.* tag triggers, in\naddition to the existing Docker manifest publish:\n\n- `build-binaries` — cross-compile wss-mux for x86_64 + aarch64 on both\n  macOS and Linux. Native runners for each arch (no QEMU). Each target\n  produces a versioned tarball + .sha256.\n- `github-release` — gh release create with all four tarballs and\n  auto-generated notes.\n- `npm-publish` — typecheck + build + test @wss-mux/client and publish\n  to npmjs.com using the NPM_TOKEN secret.\n- `homebrew-tap` — render a multi-platform formula via\n  scripts/release/render-homebrew-formula.sh and commit it to the\n  alternet-dev/homebrew-tap repo via the HOMEBREW_TAP_TOKEN secret.\n\nThe render script pulls .sha256 files from the just-uploaded GitHub\nRelease so the formula's checksums are computed from the actual\nartifacts the user will download.\n\nRequired secrets (not provisioned by this PR):\n- NPM_TOKEN — npm publish credential with @wss-mux scope access\n- HOMEBREW_TAP_TOKEN — PAT with write access to\n  alternet-dev/homebrew-tap\n\nTogether with #68 this completes the v0.5.1 patch's release plumbing.\nTagging v0.5.1 after both merge publishes wss-mux 0.5.1 to the GitHub\ncontainer registry, GitHub Releases, npmjs.com, and Homebrew tap in\nlockstep.\n\n* chore(ci): correct comment to @alternet/wss-mux-client\n\nThe package name in the npm-publish job's section comment lagged the\nrename in #73. The publish itself reads from clients/typescript/package.json\nso behaviour is unaffected; this just keeps the workflow comment honest.",
          "timestamp": "2026-05-27T00:01:33-06:00",
          "tree_id": "93f32ec38be0dcc971466aa86d7a7d6a3514b29c",
          "url": "https://github.com/alternet-dev/wss-mux/commit/e9f0f018b294c9d91b2ae0206b5a87a8fbc5d974"
        },
        "date": 1779862095761,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 307,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1890,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 4132,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33311,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 338,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2408,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28676,
            "range": "± 1911",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 399406,
            "range": "± 29052",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5178019,
            "range": "± 373860",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 157,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 144,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33569,
            "range": "± 592",
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
            "range": "± 2",
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
            "value": 381,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2875,
            "range": "± 19",
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
            "value": 21976,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 737,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 385843,
            "range": "± 3125",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6842,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 137,
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
          "id": "2f2ba9cb3b6f1ad4564a4f3dd2d8fd3cbff26c3e",
          "message": "fix(docker): COPY benches/ so cargo manifest parses (#74)\n\nThe release.yml Docker build was failing at the `cargo build --release\n--bin wss-mux` step with:\n\n    error: failed to parse manifest at `/build/Cargo.toml`\n    Caused by:\n      can't find `cbor` bench at `benches/cbor.rs` or `benches/cbor/main.rs`\n\nCargo validates every [[bench]] / [[test]] / [[bin]] target's source\npath at manifest parse time, even when the build is restricted to a\nsingle --bin target. The Dockerfile copies Cargo.toml/Cargo.lock/src/\n/tests/ but the criterion benches added in #43 live under benches/,\nwhich the Dockerfile never copied.\n\nv0.5.0 predates the benches, so this is the first tag where the gap\nmatters; v0.5.1's Docker build is the first to hit it.\n\nFix: add `COPY benches ./benches` to the Dockerfile. Cargo finds the\nsource files, manifest parses, and `--bin wss-mux` builds just the\nserver as before (no bench compilation in the image build).\n\nVerified locally with `docker build .` — succeeds.",
          "timestamp": "2026-05-27T00:22:06-06:00",
          "tree_id": "c23d5062531d93eb58c1cdea51b3be2b2e5224e6",
          "url": "https://github.com/alternet-dev/wss-mux/commit/2f2ba9cb3b6f1ad4564a4f3dd2d8fd3cbff26c3e"
        },
        "date": 1779863339993,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 322,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1908,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 4236,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33287,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 332,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2398,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28635,
            "range": "± 1550",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 405091,
            "range": "± 29120",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 6670789,
            "range": "± 663554",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 159,
            "range": "± 466",
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
            "value": 33573,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 165,
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
            "value": 316,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 100,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 4151,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 148,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 38346,
            "range": "± 1471",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 740,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 373445,
            "range": "± 1346",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6534,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 137,
            "range": "± 3",
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
          "id": "f1902a02945cf491e3b32a9a922448c43475be70",
          "message": "chore: bump version to 0.5.2 (#75)\n\nv0.5.1 release run hit two issues: the npm-publish job needed a token\nwith 2FA-bypass (now in place), and the Docker amd64 build failed at\ncargo manifest parse because the Dockerfile didn't COPY benches/ (fixed\nin #74). Both fixes are on trunk; this bump cuts v0.5.2 as the clean\nrelease that picks them up.\n\nSame lockstep bump on the TypeScript SDK: @alternet/wss-mux-client@0.5.2.",
          "timestamp": "2026-05-27T00:28:01-06:00",
          "tree_id": "2cbf3ab80325705b0bdca041943040aae6fc3e83",
          "url": "https://github.com/alternet-dev/wss-mux/commit/f1902a02945cf491e3b32a9a922448c43475be70"
        },
        "date": 1779863692102,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 253,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1612,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3669,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 28763,
            "range": "± 810",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 329,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2377,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29304,
            "range": "± 1743",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 385167,
            "range": "± 28812",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5350802,
            "range": "± 383180",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 144,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31660,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 160,
            "range": "± 6",
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
            "value": 74,
            "range": "± 0",
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
            "value": 397,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 102,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3948,
            "range": "± 82",
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
            "value": 41841,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 664,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 364578,
            "range": "± 3311",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7074,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 134,
            "range": "± 2",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}