window.BENCHMARK_DATA = {
  "lastUpdate": 1780335642306,
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
          "id": "61abdb49618d6b9e4f4893c20081e24b9c55907c",
          "message": "chore(ci): switch npm-publish to Trusted Publishing via OIDC (#76)\n\nReplaces the static NPM_TOKEN-based publish with npm's Trusted\nPublishing — the runner mints a short-lived OIDC token that npm\nexchanges for publish capability via the trusted publisher configured\non the @alternet/wss-mux-client package settings page.\n\nChanges to the npm-publish job:\n\n- Add `permissions: id-token: write` so GitHub Actions issues the\n  workflow an OIDC identity token.\n- Bump `node-version` from \"20\" to \"22\". npm Trusted Publishing\n  requires npm >= 11.5; Node 22 ships npm 11.x by default, whereas\n  Node 20's bundled npm 10.x is too old.\n- Change `npm publish` to `npm publish --provenance --access public`.\n  --provenance attaches a signed attestation linking the published\n  version to this workflow run (visible as a \"Provenance\" badge on\n  npmjs.com); it's required when publishing via Trusted Publishing.\n- Drop the NODE_AUTH_TOKEN env var. The static NPM_TOKEN secret is\n  no longer used by this job.\n\nAfter this lands and the next release tag publishes successfully via\nOIDC, the NPM_TOKEN repo secret can be deleted and the corresponding\nclassic automation token on npmjs.com revoked.",
          "timestamp": "2026-05-27T00:48:22-06:00",
          "tree_id": "8a66c2e509f9d597f5206b808548252fcd477647",
          "url": "https://github.com/alternet-dev/wss-mux/commit/61abdb49618d6b9e4f4893c20081e24b9c55907c"
        },
        "date": 1779864912405,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 248,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1651,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3782,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29179,
            "range": "± 549",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 327,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2366,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29215,
            "range": "± 1373",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 365462,
            "range": "± 26824",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3894051,
            "range": "± 203775",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 139,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 133,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31914,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 158,
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
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 335,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 99,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2277,
            "range": "± 11",
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
            "value": 43160,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 659,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 382746,
            "range": "± 9931",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7091,
            "range": "± 133",
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
          "id": "5fc5d7149e002e5805189f43a527fbb08781bd0b",
          "message": "ci(release): create github-release job to auto-release on tags (#77)\n\nThree changes to the github-release job, matching what alternet-dev/wavefront\nalready does in its release.yml:\n\n1. **--notes-from-tag instead of --generate-notes.** The release body\n   now comes from the annotation message of the `git tag -a` that\n   triggered the workflow, rather than an auto-summary of commit\n   messages. This makes the release-page content match what we already\n   write into tag annotations (e.g. v0.5.2's annotation describing the\n   wire change + SDK introduction). Encourages writing real release\n   notes at tag time rather than scraping the commit log.\n\n2. **Idempotent.** Re-running the workflow on the same tag (e.g. after\n   a partial failure where binaries built but the release create call\n   errored) no longer fails on \"release already exists\" — it uploads\n   the binaries to the existing release with --clobber. The original\n   create path stays unchanged for first runs.\n\n3. **--verify-tag.** gh release create refuses to create a release for\n   a tag that doesn't actually exist in the repo, which guards against\n   the edge case of the workflow being triggered against a deleted tag.\n\nPlus prerelease detection: any tag containing `-` (e.g. `v1.0.0-rc1`,\n`v0.6.0-beta.2`) gets `--prerelease`. No-op for the current `vX.Y.Z`\nrelease line but unblocks pre-release tagging without a workflow edit.\n\nNo new dependencies; the job still uses `gh release` + `actions/checkout`\n+ `actions/download-artifact`.\n\n(Originally drafted as a release-please integration; the wavefront\npattern — write your own annotated tag, workflow auto-creates the\nrelease page from it — is simpler and is the established pattern in\nthis org.)",
          "timestamp": "2026-05-27T01:40:48-06:00",
          "tree_id": "0dada37f5d8aec7be2dacc1895a07e27e10b2ca8",
          "url": "https://github.com/alternet-dev/wss-mux/commit/5fc5d7149e002e5805189f43a527fbb08781bd0b"
        },
        "date": 1779868047952,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 255,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1641,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3698,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29217,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 328,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2378,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29300,
            "range": "± 1422",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 370789,
            "range": "± 27967",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5446038,
            "range": "± 982151",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 137,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31365,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 159,
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
            "range": "± 1",
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
            "value": 329,
            "range": "± 4",
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
            "value": 3830,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 149,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 21173,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 666,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 367898,
            "range": "± 2680",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6959,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 135,
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
          "id": "0831285e6d2e87dd0b9fc0e1091da0d07f58fcde",
          "message": "ci(release): switch homebrew-tap to GitHub App auth (match wavefront) (#78)\n\nReplaces the static HOMEBREW_TAP_TOKEN PAT with the GitHub App token\nflow already used by alternet-dev/wavefront. The App mints a short-\nlived token (~1 hour) scoped to just the homebrew-tap repo, then the\ncheckout + push use that token instead of a long-lived PAT.\n\nSetup required at org level (already present, per wavefront's usage):\n\n- vars.HOMEBREW_TAP_APP_ID — the App's numeric id (non-sensitive).\n- secrets.HOMEBREW_TAP_APP_PRIVATE_KEY — the App's signing key (PEM).\n- wss-mux added to the App's installed repositories AND to the access\n  list for the org variable/secret.\n\nResolves and uses the App's bot user identity for the commit (matching\nwavefront's pattern: bot user id resolved via GitHub API, then used\nin `git config user.email`). The committer attribution reflects the\nApp, not the generic github-actions[bot] identity.\n\nAfter this lands and the next release succeeds, the repo-level\nHOMEBREW_TAP_TOKEN secret can be deleted.",
          "timestamp": "2026-05-27T01:47:34-06:00",
          "tree_id": "b38c1d2803d8ecf913c62d64c6fa989207cd7f39",
          "url": "https://github.com/alternet-dev/wss-mux/commit/0831285e6d2e87dd0b9fc0e1091da0d07f58fcde"
        },
        "date": 1779868465277,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 317,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1928,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3967,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33592,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 330,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2360,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28323,
            "range": "± 1802",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 416708,
            "range": "± 32287",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4874380,
            "range": "± 553921",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 160,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 140,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33250,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 167,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 17,
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
            "value": 80,
            "range": "± 2",
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
            "value": 334,
            "range": "± 1",
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
            "value": 3746,
            "range": "± 20",
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
            "value": 38365,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 374695,
            "range": "± 10488",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6327,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 136,
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
            "email": "evan@macgregor.llc",
            "name": "Evan MacGregor",
            "username": "evan-macgregor"
          },
          "distinct": true,
          "id": "5b9e2831b2aa27daed8e869ba43ac7c1c49265be",
          "message": "ci(release): switch homebrew-tap to GitHub App auth (#78)\n\nReplaces the static HOMEBREW_TAP_TOKEN PAT with the GitHub App token\nflow. The App mints a short-lived token (~1 hour) scoped to just the\nhomebrew-tap repo, then the checkout + push use that token instead of\na long-lived PAT.\n\nSetup required at org level (already present):\n\n- vars.HOMEBREW_TAP_APP_ID — the App's numeric id (non-sensitive).\n- secrets.HOMEBREW_TAP_APP_PRIVATE_KEY — the App's signing key (PEM).\n- wss-mux added to the App's installed repositories AND to the access\n  list for the org variable/secret.\n\nResolves and uses the App's bot user identity for the commit: bot\nuser id resolved via GitHub API, then used in `git config user.email`.\nThe committer attribution reflects the App, not the generic\ngithub-actions[bot] identity.\n\nAfter this lands and the next release succeeds, the repo-level\nHOMEBREW_TAP_TOKEN secret can be deleted.",
          "timestamp": "2026-05-27T01:51:01-06:00",
          "tree_id": "b38c1d2803d8ecf913c62d64c6fa989207cd7f39",
          "url": "https://github.com/alternet-dev/wss-mux/commit/5b9e2831b2aa27daed8e869ba43ac7c1c49265be"
        },
        "date": 1779868789421,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 250,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1461,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3180,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 26512,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 349,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 3118,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 35581,
            "range": "± 1403",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 515853,
            "range": "± 26463",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 6239852,
            "range": "± 366636",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 147,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 122,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 36384,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 145,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 64,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 408,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 4122,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 40773,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 685,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 414898,
            "range": "± 1589",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 5944,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 121,
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
          "id": "d7490e8daaf37d97cf80953edad64f6a46df5d24",
          "message": "feat(wire)!: drop version from URL routes (#55) (#79)\n\nBREAKING CHANGE: the HTTP/WebSocket routes lose their `/v1/` prefix.\nA v0.5.x client posting to `/v1/events` or upgrading at `/v1/stream`\nnow gets HTTP 404; consumers must post to `/events`, `/events/batch`,\nand upgrade at `/stream`. The peer-relay endpoint moves from\n`/internal/v1/relay` to `/internal/relay` (relevant only to operators\nconfiguring NetworkPolicies or proxies).\n\nThis is the sister change to the subprotocol-version drop that\nshipped in v0.5.1/v0.5.2 (#72) — both are part of #55's argument\nthat pre-1.0 lockstep versioning makes URL/subprotocol version fields\nredundant. Closes #55.\n\nRoute changes:\n\n  /v1/events            → /events\n  /v1/events/batch      → /events/batch\n  /v1/stream            → /stream\n  /internal/v1/relay    → /internal/relay\n\n## Surface touched\n\n- `src/server/mod.rs` — Router registrations.\n- `src/server/relay.rs` — outbound POST URL to peer's relay endpoint.\n- `src/peers/url.rs` — doc comment referencing the endpoint.\n- `src/bin/loadgen/{client,server,scenarios/mod}.rs` — loadgen client\n  URL building + REST surface comment.\n- `tests/integration/**.rs` — every integration test that posts to\n  the push endpoint or connects to the WS stream (~15 files).\n- `docs/{architecture,operations,embedding}.md` — every prose\n  reference to the routes.\n- `README.md` — surface-area mention.\n\nDownstream consumers (e.g. application-template's publisher) need\nto update their POST URLs before consuming v0.5.3+.\n\n## Verification\n\n- `cargo build --all-targets` — green.\n- `cargo test --all-targets` — all 262 tests pass on the new paths.\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.\n\n## Companion changes that are already done\n\n- Subprotocol drop (#72, v0.5.1/v0.5.2).\n- TS SDK was always path-agnostic via the consumer-supplied wssUrl —\n  no SDK code change needed.\n\n## What still needs to happen for the consumer\n\nAfter this lands and v0.5.3 ships, downstream callers need to update\ntheir POST URLs (the SDK's wssUrl is operator-set so the WS path is\nsimilarly operator-configurable from the SDK's perspective).",
          "timestamp": "2026-05-27T01:55:18-06:00",
          "tree_id": "5446ba50e46cf97cdab22ebd6c6e8a07e82d3d28",
          "url": "https://github.com/alternet-dev/wss-mux/commit/d7490e8daaf37d97cf80953edad64f6a46df5d24"
        },
        "date": 1779868943091,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 253,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1614,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3768,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29275,
            "range": "± 105",
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
            "value": 2432,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29357,
            "range": "± 1682",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 336841,
            "range": "± 22678",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 7787556,
            "range": "± 987299",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 146,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 133,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31709,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 158,
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
            "value": 357,
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
            "value": 2311,
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
            "value": 21108,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 657,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 367535,
            "range": "± 3683",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6849,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 134,
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
          "id": "690cd579f025722ff0430e5e93705f4113724453",
          "message": "chore: bump version to 0.5.3 (#80)\n\nCuts v0.5.3, which ships the URL-path version drop (#55, merged in\n#79). Wire-breaking: `/v1/events`, `/v1/events/batch`, `/v1/stream`,\nand `/internal/v1/relay` all lose the `/v1/` prefix — a v0.5.2 client\nposting to `/v1/events` against a v0.5.3 server gets HTTP 404.\n\nCombined with the subprotocol-version drop already in v0.5.1/v0.5.2,\nthis completes #55's argument: pre-1.0 lockstep versioning makes the\nURL and subprotocol version fields both redundant, so both are gone.\n\nLockstep bump on the TypeScript SDK: @alternet/wss-mux-client@0.5.3.\nThe SDK is path-agnostic via its wssUrl constructor option, so no\ncode change is needed on the client side — only the consumer's\nconfigured URL.",
          "timestamp": "2026-05-27T01:57:36-06:00",
          "tree_id": "17bdf584e95692cd403e0ae19d6878e163e98917",
          "url": "https://github.com/alternet-dev/wss-mux/commit/690cd579f025722ff0430e5e93705f4113724453"
        },
        "date": 1779869058006,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 252,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1638,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3673,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29295,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 334,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2402,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29471,
            "range": "± 1764",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 322703,
            "range": "± 6552",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4626452,
            "range": "± 364509",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 138,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 136,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 32405,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 158,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 17,
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
            "range": "± 1",
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
            "value": 347,
            "range": "± 1",
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
            "value": 2277,
            "range": "± 10",
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
            "value": 37800,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 667,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 365264,
            "range": "± 3294",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6566,
            "range": "± 47",
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
          "id": "3e6a596e012387ba2ba27d17e3d2b1cf4c781704",
          "message": "fix(routes)!: drop the `z` from health/ready probes (#81)\n\nBREAKING (operator-visible): liveness/readiness probe routes lose\ntheir stylistic `z` suffix.\n\n  /healthz → /health\n  /readyz  → /ready\n\nThe `*z` convention is borrowed from Google's internal `/varz`-style\npages; outside that lineage it's pure noise. Now that we're also\ndropping URL-path versioning (`/v1/...` in #79), it makes sense to\nclean the rest of the public route surface at the same time.\n\nOperator impact: Kubernetes liveness/readiness probes pointed at\n`/healthz`/`/readyz` need their paths updated to `/health`/`/ready`.\nThe docs/operations.md Deployment manifest in the repo is already\nupdated.\n\nSurface touched:\n\n- `src/server/mod.rs` — Router registrations and handler functions.\n- `tests/integration/{healthz.rs,readyz.rs}` renamed to\n  `{health.rs,ready.rs}` (file + module name + test function names).\n- `tests/integration/main.rs` — module registrations.\n- `src/main.rs`, `docs/{architecture,operations,embedding}.md`, and\n  `README.md` — prose references.\n\n## Verification\n\n- `cargo build --all-targets` — green.\n- `cargo test --all-targets` — all tests pass on the new paths.\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.\n\nShips as part of v0.5.3 (still publishing — see in-flight tag).",
          "timestamp": "2026-05-27T02:26:30-06:00",
          "tree_id": "98eb44bcc3512ab18105417fdc439522159415b6",
          "url": "https://github.com/alternet-dev/wss-mux/commit/3e6a596e012387ba2ba27d17e3d2b1cf4c781704"
        },
        "date": 1779870804525,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 245,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1632,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3923,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29379,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 325,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2381,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29623,
            "range": "± 2100",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 343218,
            "range": "± 30918",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3648915,
            "range": "± 171588",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 138,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 133,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31717,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 158,
            "range": "± 8",
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
            "range": "± 1",
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
            "value": 355,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 101,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2297,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 147,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 42743,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 666,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 363726,
            "range": "± 3042",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7317,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 135,
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
          "id": "fb8de3b3b09c62815b3f25049c402fdd2e06a834",
          "message": "fix(ci): pin npm to >=11.5.1 for Trusted Publishing support (#82)\n\nThe npm-publish job has been failing on v0.5.3 with HTTP 404 on the\nregistry PUT, despite OIDC token signing working (provenance landed in\nSigstore each attempt). Root cause: npm Trusted Publishing (token-free\nPUT via OIDC) requires npm >= 11.5.1, but `setup-node@v4 node-version:\n\"22\"` ships whichever npm Node 22.x bundles — often npm 10.x, which has\n`--provenance` (added in 9.5) but lacks the registry PUT auth flow that\ntrusts an OIDC token in place of NODE_AUTH_TOKEN. The publish signs\nprovenance, then PUTs without auth, the registry 404s with the\nmisleading \"not in this registry\" message.\n\nTwo changes to the npm-publish job:\n\n- Bump node-version 22 → 24. Node 24 ships current npm by default\n  (11.x in early 2025+). The earlier comment about Node 22 having\n  npm 11 was wrong on the specific patch versions that resolve.\n\n- Explicitly `npm install -g 'npm@>=11.5.1'` before publish, with\n  `node --version` + `npm --version` logged. Belt-and-suspenders so\n  the publish never silently uses a stale npm; diagnostic versions\n  in the log if anything else regresses.\n\nVerified against the v0.5.3 release log: provenance is generated and\npushed to Sigstore (https://search.sigstore.dev/?logIndex=1640898082),\nthen PUT fails — the exact signature of \"npm CLI version too old for\nTrusted Publishing.\" Configuration on the @alternet/wss-mux-client\ntrusted publisher page is correct (GitHub Actions / alternet-dev /\nwss-mux / release.yml / no environment / allow `npm publish`).",
          "timestamp": "2026-05-27T10:07:41-06:00",
          "tree_id": "835cfb11c2b858bd8485e75cb6b7fb822dbd3734",
          "url": "https://github.com/alternet-dev/wss-mux/commit/fb8de3b3b09c62815b3f25049c402fdd2e06a834"
        },
        "date": 1779898472722,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 250,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1458,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3218,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 26314,
            "range": "± 641",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 350,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 3116,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 35927,
            "range": "± 1895",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 515588,
            "range": "± 21896",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 7556604,
            "range": "± 738200",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 149,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 121,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 35827,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 141,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 411,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 4145,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 121,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 40566,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 685,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 418257,
            "range": "± 1605",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 5943,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 123,
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
          "id": "df3b2ec03f2144905ef76424153bc049368af41d",
          "message": "build: drop Intel-Mac (x86_64-apple-darwin) target (#84)\n\nGitHub's free-tier macos-13 runners are persistently backed up — the\nv0.5.3 release run sat with the Intel-Mac binary build queued for\nhours, blocking the downstream github-release and homebrew-tap jobs\nsince both `needs: build-binaries`.\n\nCross-compiling x86_64-apple-darwin from macos-14 (arm64) is an\noption, but the Intel-Mac audience for a Rust server binary is\neffectively nil at this point: ARM Macs have been the default for\n4+ years and server-side deployments use Linux containers via the\nDocker image regardless. Carrying the target costs CI minutes,\nadds a moving piece to maintain, and serves nobody we can name.\n\nChanges:\n\n- `.github/workflows/release.yml` — drop the x86_64-apple-darwin\n  matrix entry from build-binaries. Three targets remain:\n  aarch64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu.\n\n- `scripts/release/render-homebrew-formula.sh` — drop the `on_intel do`\n  block inside `on_macos do`. Homebrew on Intel Macs will report\n  \"no available formula\"; Intel-Mac users who need wss-mux can\n  `cargo install --git https://github.com/alternet-dev/wss-mux`\n  from source.\n\nIf demand materializes later, the right re-introduction is cross-compile\nfrom macos-latest using `rustup target add x86_64-apple-darwin` — no\nnew runner type needed.",
          "timestamp": "2026-05-27T11:05:42-06:00",
          "tree_id": "89ba6efcf19be664ee01fccc85f3dae23d5c2ece",
          "url": "https://github.com/alternet-dev/wss-mux/commit/df3b2ec03f2144905ef76424153bc049368af41d"
        },
        "date": 1779901919164,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 227,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1558,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3087,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 25912,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 255,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 1837,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 22358,
            "range": "± 1559",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 359150,
            "range": "± 24910",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 7524271,
            "range": "± 620615",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 133,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 109,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 25903,
            "range": "± 425",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 126,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 259,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 1758,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 30269,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 568,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 299659,
            "range": "± 2126",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 4897,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 110,
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
          "id": "3dee99e4b030b220d901f4efa051ff35ee63a333",
          "message": "ci(release): make npm-publish idempotent on retags (#85)\n\nIf a release run re-fires for a tag whose version is already published\nto npm, the job currently 403s. That's the only non-idempotent piece of\nthe release pipeline — github-release became idempotent in #77 and the\nHomebrew tap update is a no-op when the formula content matches what's\nalready on the tap.\n\nAdd a pre-publish check: query the registry for the current version, and\nskip the publish if it's already there. Behaviour:\n\n- First run for a version: skip falls through, publish proceeds as before.\n- Retag for an already-published version: skip prints a message and the\n  job exits 0. Downstream jobs (none currently) would still run.\n\nWhy this matters: when a release run partially succeeds (e.g. the Mac\nbinary build queues forever and the run gets cancelled before the GH\nRelease page + Homebrew tap get created), retagging is the natural way\nto re-fire the workflow. Before this change, the npm-publish job would\nfail on retag because the npm version had already been published in the\nprior partial run. Now the retag completes cleanly across the board.\n\nVerified by inspection: `npm view <pkg>@<version> version` returns the\nversion string on success (exit 0) and a 404 on missing (exit 1).\n`>/dev/null 2>&1` swallows both the success stdout and the 404 stderr,\nso the `if` branches on exit code alone.",
          "timestamp": "2026-05-27T12:16:35-06:00",
          "tree_id": "8c573609f9bfb0620fe9c9deb55a26c7386e367d",
          "url": "https://github.com/alternet-dev/wss-mux/commit/3dee99e4b030b220d901f4efa051ff35ee63a333"
        },
        "date": 1779906209210,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 325,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1900,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 4101,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33661,
            "range": "± 1006",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 329,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2371,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28615,
            "range": "± 1939",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 424357,
            "range": "± 30273",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4634998,
            "range": "± 428512",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 160,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 141,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33758,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 171,
            "range": "± 3",
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
            "value": 78,
            "range": "± 1",
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
            "value": 353,
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
            "value": 2842,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 149,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 38683,
            "range": "± 3156",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 739,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 376360,
            "range": "± 3025",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6715,
            "range": "± 17",
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
          "id": "438a9d0f3daf3c48141da55c993b6782a4d9773f",
          "message": "feat(wire): add Publish ClientFrame variant + manifest publish audience (#87)\n\nFirst PR in the WS-publish workstream. Adds the wire schema for the new\n`publish` action and the matching manifest field; leaves the server-\nside dispatch for a follow-up so the wire-level change can land in\nisolation.\n\n## Wire surface\n\n`ClientFrame::Publish { id, stream, key?, payload }` joins the existing\n`Auth`/`Subscribe`/`Unsubscribe` variants. Serde-tagged the same way;\n`key` is optional with the same skip-if-none convention as `subscribe`.\n\nThere is no success ack — same ack-by-absence pattern as subscribe.\nErrors are returned as `error` frames with `id` echoed; new keep-open\nerror codes are documented in `docs/protocol.md`:\n\n- `unauthorized_publish`\n- `publish_payload_too_large`\n- `unknown_stream` (reused; was subscribe-only)\n- `rate_limited` (reused)\n\n## Manifest — rename to action-keyed audiences\n\nRestructure the stream's two principal audiences as actions rather than\nabstract role-nouns. Old single `audience` field becomes `subscribe`;\nthe new write-side audience is `publish`:\n\n  audience  →  subscribe   (read)\n  (new)     →  publish     (write, WS only)\n\nThe field names match the protocol verbs (`ClientFrame::Subscribe` /\n`ClientFrame::Publish`) so the mental model is single across the wire\nand the manifest. Both fields are lists of principal patterns with the\nsame matching rules (exact, trailing-`*` prefix, bare `*`).\n\n`subscribe` is required and must be non-empty (validated at load).\n`publish` defaults to empty ⇒ no one may WS-publish to that stream\nuntil explicit opt-in. HTTP `POST /events` is unaffected — it uses the\nshared `WSS_MUX_PUSH_AUTH_TOKEN`, not principal audience. `publish`\ngates the WS `publish` frame only.\n\n### Manifest migration\n\nExisting manifests using `audience: [...]` must rename to\n`subscribe: [...]`. Pre-1.0 wire-licensed break, documented here:\n\n  streams:\n    - stream: chat_messages\n      subscribe: [role:member]      # was: audience\n      publish:   [role:member]      # new, opt-in\n\nManifestError variant `EmptyAudience` is renamed `EmptySubscribe`.\nThe `audience_admits()` helper keeps its name — it's a generic\nalgorithm operating on a principal-pattern set, not on the renamed\nfield specifically.\n\n## Server-side stub\n\nThe reader-loop match in `src/server/ws.rs` gets an exhaustive\n`ClientFrame::Publish` arm that emits `unknown_frame_type` (close 4400)\nuntil the dispatch path is wired in.\n\n## Docs\n\n- `docs/protocol.md` — new `publish` frame section; error-codes table\n  updated.\n- `docs/embedding.md` — §3 \"Declaring streams and audiences\" YAML\n  examples updated to use `subscribe`; new \"Publish audience (WS\n  publish)\" subsection documenting the parallel `publish` field.\n\n## Why `ClientFrame` lost `Eq`\n\n`Publish` carries `serde_json::Value`, which doesn't implement `Eq`.\n`ClientFrame`'s derive list drops `Eq` accordingly. No callers depend\non it.\n\n## Verification\n\n- `cargo test --all-targets` — 271 tests pass (was 262 on trunk; +9 new).\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.",
          "timestamp": "2026-05-28T17:13:17-06:00",
          "tree_id": "05dfa91cde58ceceee27a4c09dc3718132c127c0",
          "url": "https://github.com/alternet-dev/wss-mux/commit/438a9d0f3daf3c48141da55c993b6782a4d9773f"
        },
        "date": 1780010449628,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 251,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1623,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3777,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29493,
            "range": "± 910",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 324,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2491,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 30984,
            "range": "± 1853",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 376260,
            "range": "± 26781",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3956474,
            "range": "± 205763",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 138,
            "range": "± 3",
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
            "value": 31897,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 162,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 337,
            "range": "± 6",
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
            "value": 3766,
            "range": "± 11",
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
            "value": 42814,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 663,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 365004,
            "range": "± 4048",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6891,
            "range": "± 58",
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
          "id": "f701778d604254bffa3fdbac7f5336b25a254025",
          "message": "feat(server): handle Publish frame (audience + dispatch) (#88)\n\nReplace the stub `unknown_frame_type` arm with the real WS-publish\nhandler. The publish path shares the dispatch path with HTTP push:\nonce the frame passes the wire/audience/cap checks, it goes through\n`dispatch` (registry match → per-sub channel) and `spawn_relay`\n(fire-and-forget peer fanout), identical to `accept_events` in\n`src/server/http.rs`.\n\nThe only WS-specific surface on top of the shared dispatch is the\n`publish`-audience check against the connection's principals.\n\n## Handler logic\n\nFor a `ClientFrame::Publish { id, stream, key, payload }`:\n\n1. If not authenticated, emit `unauthenticated` (echo id) and close\n   4401 — same posture as a pre-auth `subscribe`.\n2. Look up the stream in the manifest. Absent ⇒ `unknown_stream`\n   (keep-open, echo id).\n3. Check the connection's principals against the stream's `publish`\n   audience. No match ⇒ `unauthorized_publish` (keep-open, echo id).\n4. If the stream sets `max_payload_bytes`, measure the payload's JSON\n   serialization (stable across producer/relay wire codecs, identical\n   to HTTP push's cap enforcement). Over ⇒ `publish_payload_too_large`\n   (keep-open, echo id).\n5. Build an `EventEnvelope` and call `dispatch(..., Producer)` for\n   local registry fanout + `spawn_relay(...)` for one-hop peer fanout.\n\n## Parser whitelist\n\nBoth `parse_client_frame_json` and `parse_client_frame_cbor` had an\nexplicit type-string whitelist (`auth | subscribe | unsubscribe`)\nthat fell through to `unknown_frame_type` for anything else. Added\n`publish` so well-formed publish frames reach the dispatch arm.\n\n## New ProtocolError variants\n\nTwo new keep-open variants matching the wire docs from PR-1:\n\n- `UnauthorizedPublish { id }`\n- `PublishPayloadTooLarge { id }`\n\nBoth echo the publish frame's `id`. `code()`, `message()`, and `id()`\nimplementations follow the existing pattern.\n\n`UnauthorizedSubscribe`'s message string is rewritten (\"...stream\nsubscribe audience\") to align with the field rename in PR-1; the wire\ncode string is unchanged.\n\n## Tests\n\nNew `tests/integration/publish.rs` covering:\n\n- Happy path: publish → own subscriber receives the event.\n- Default-deny (`publish` absent in manifest) ⇒ `unauthorized_publish`.\n- Principal mismatch (`publish: [role:operator]` vs `role:member`).\n- Unknown stream.\n- Payload over `max_payload_bytes` cap.\n- Payload under cap ⇒ dispatched.\n- Pre-auth publish closes 4401.\n\n`mod publish` added to `tests/integration/main.rs`.\n\n## Verification\n\n- `cargo test --all-targets` — 278 tests pass (was 271 on trunk; +7 new).\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.",
          "timestamp": "2026-05-28T17:23:01-06:00",
          "tree_id": "82be15633539a01052d0418104945590ceb589d1",
          "url": "https://github.com/alternet-dev/wss-mux/commit/f701778d604254bffa3fdbac7f5336b25a254025"
        },
        "date": 1780010992705,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 248,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1615,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3677,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29635,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 359,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2414,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29062,
            "range": "± 1601",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 334810,
            "range": "± 24320",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4663708,
            "range": "± 375005",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 136,
            "range": "± 588",
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
            "value": 32912,
            "range": "± 1055",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 157,
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
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 77,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 392,
            "range": "± 2",
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
            "value": 3936,
            "range": "± 38",
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
            "value": 42298,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 663,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 366344,
            "range": "± 3896",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7025,
            "range": "± 92",
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
          "id": "f8d7012258fc0d9dd88bd98b7c3a19c0978bd14b",
          "message": "feat(ratelimit): add PerSourceRateLimiter + idle eviction (#89)\n\nFoundation for per-source rate limiting on WS publish (and eventually\non HTTP push). Builds on the existing `TokenBucket` rather than\nduplicating it — every source gets its own bucket; the pool just\nmanages keyed lookup, lazy insert, and idle eviction.\n\nThe shape:\n\n  pub struct PerSourceRateLimiter {\n      buckets: DashMap<String, Mutex<BucketEntry>>,\n      capacity: u32,\n      refill_per_sec: u32,\n      idle_ttl: Duration,\n  }\n\n`try_take(&self, source, now)` returns the same `bool` admit/reject\ncontract as `TokenBucket::try_take`. The `now` is also recorded as the\nentry's `last_seen` so the idle sweep can find stale entries.\n\n`sweep_idle(&self, now)` drops entries whose `last_seen` is older than\n`now - idle_ttl` and returns the eviction count. Intended for a low-\nfrequency tokio interval task, not the publish hot path.\n\n`active_sources()` returns the tracked-source count, surfaced later as\na Prometheus gauge.\n\n## Hot-path properties\n\n- **DashMap** for sharded concurrent access — no global Mutex.\n- **Per-bucket Mutex** held only for the refill + take, not for the\n  DashMap lookup.\n- **Get-or-insert with race-safe fallback** — if two connections from a\n  newly-seen source race the first `try_take`, the DashMap `Entry` API\n  resolves the race without double-admitting or double-counting.\n- **First-take admits when capacity > 0** — a fresh source starts with\n  a full burst (matches `TokenBucket::new`).\n\n## Tests\n\nSeven new unit tests in `src/ratelimit.rs`:\n\n- Distinct sources are isolated (one's bucket doesn't touch another's).\n- Same source key shared across \"connections\" shares one bucket\n  (the pool's keying contract — two WS sessions with the same JWT\n  `sub` share the budget).\n- Refill over elapsed time.\n- First-take from a never-seen source admits the full burst.\n- Sweep evicts entries past TTL while preserving recently-touched ones.\n- Sweep with `now == last_seen` for every entry leaves them alone\n  (off-by-one guard on the cutoff math).\n- Zero-capacity rejects the first publish from every source.\n\n## Bench\n\n`benches/per_source_rate_limit.rs` covers the publish-path hot loop:\n\n- Steady-state, single source (warm-path lookup + take).\n- Steady-state mixed across N sources (10, 100, 1k, 10k) — picks up\n  any DashMap shard contention the single-source bench can't.\n- Cold insert (first take for a never-seen source).\n- Sweep cost vs map size (100, 1k, 10k entries, no eviction — the\n  realistic baseline).\n\nLocal quick-run numbers:\n\n- per_source_try_take_hit:         ~145 ns\n- per_source_try_take_cold_insert: ~580 ns\n- per_source_sweep_idle/10k:       ~80 µs\n\nRegistered as a new `[[bench]]` entry in `Cargo.toml`.\n\n## Verification\n\n- `cargo test --all-targets` — 285 tests pass (was 278 on trunk; +7 new).\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.\n- `cargo bench --bench per_source_rate_limit -- --quick` — runs.",
          "timestamp": "2026-05-28T18:03:16-06:00",
          "tree_id": "f22dceb6994a938e5813bfc3d6eefe8d5a887488",
          "url": "https://github.com/alternet-dev/wss-mux/commit/f8d7012258fc0d9dd88bd98b7c3a19c0978bd14b"
        },
        "date": 1780013516215,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 307,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1879,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3917,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33261,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 331,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2441,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 31179,
            "range": "± 1870",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 385707,
            "range": "± 35481",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4464523,
            "range": "± 874809",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 160,
            "range": "± 11",
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
            "value": 33315,
            "range": "± 739",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 166,
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
            "name": "per_source_try_take_hit",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 129,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 148,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 338,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 354,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1906,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 21326,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 505,
            "range": "± 5",
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
            "value": 3904,
            "range": "± 25",
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
            "value": 39004,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 737,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 386134,
            "range": "± 1324",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6563,
            "range": "± 32",
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
          "id": "eb636a5e7c4ef83503b8aa96e645d6600f582378",
          "message": "feat(server): per-source rate limit on WS publish (#90)\n\nPlumbs the PerSourceRateLimiter from #89 through `AppState` and the WS\npublish handler. The wire-visible effect: a WS publish frame that\nexceeds the source's bucket is rejected with a keep-open `rate_limited`\nerror frame (echoing the publish `id`). Source key derivation lets two\nWS sessions with the same JWT `sub` share one budget; a missing `sub`\nfalls back to a per-connection key (or is refused outright in strict\nmode).\n\n## Config\n\nFour new env vars, all inert at default:\n\n- `WSS_MUX_WS_PUBLISH_RATE` (default `0` — feature off). When `0`, no\n  limiter is constructed, no sweep task is spawned, the handler arm\n  short-circuits past the rate-limit branch — byte-identical to the\n  pre-feature behaviour.\n- `WSS_MUX_WS_PUBLISH_BURST` (default = `2 * RATE` when set, else `0`).\n- `WSS_MUX_WS_PUBLISH_REQUIRE_SUB` (default `false`). When `true`, a\n  token whose `sub` claim is empty is refused at publish time with\n  `unauthorized_publish` rather than falling back to a per-connection\n  bucket. Operators who want \"no anonymous publishers\" set this.\n- `WSS_MUX_WS_PUBLISH_IDLE_TTL_SECS` (default `300`). Sweep cadence is\n  one tenth this.\n\nA new `parse_bool` helper resolves common truthy/falsy spellings; a new\n`ConfigError::InvalidBool` rejects garbage with a friendly message.\n\n## AppState\n\nNew `ws_publish_limiter: Option<Arc<PerSourceRateLimiter>>` field,\nconstructed in `try_new` iff the config's `ws_publish_rate_per_sec > 0`.\nThe `Arc` lets the same instance be handed to the idle-eviction task\nwithout aliasing the AppState's inner Arc.\n\n## Handler\n\nIn `src/server/ws.rs`:\n\n- The auth arm now also captures `claims.sub` into a connection-local\n  `auth_sub: Option<String>` (alongside the existing `principals`).\n- The publish arm, after the audience and payload-cap checks but before\n  dispatch, consults the limiter when configured:\n\n  Source key resolution:\n\n    JWT sub non-empty                   ⇒ `sub:<sub>`\n    JWT sub empty AND !require_sub      ⇒ `conn:<conn_id>`\n    JWT sub empty AND  require_sub      ⇒ unauthorized_publish\n\n  A rejected publish increments `wss_mux_ws_publish_rate_limited` and\n  emits `rate_limited` (keep-open, echo id). The dispatcher and the\n  peer-relay are not touched.\n\n## Metrics\n\n`wss_mux_ws_publish_rate_limited` counter — no source label\n(cardinality risk).\n\n## Idle eviction task\n\n`src/main.rs` gains `spawn_ws_publish_idle_sweeper(state)`, spawned in\nthe boot sequence next to the peer refresher and the JWKS refresher.\nInert (task not spawned) when the limiter is disabled. Cadence is\n`idle_ttl / 10`, with the tokio interval set to `MissedTickBehavior::\nDelay` so a stalled scheduler doesn't burst-fire missed ticks. Each\nsweep logs the eviction count at `debug` level when non-zero.\n\n## Tests\n\n### Config (`src/config.rs`)\n\nSix new unit tests covering the four env vars + their defaults +\nparse_bool's truthy/falsy/garbage handling.\n\n### Integration (`tests/integration/publish_rate_limit.rs`)\n\nSeven end-to-end tests against a real server:\n\n- `flood_from_one_source_eventually_rate_limited` — burst exhausted,\n  next publish gets `rate_limited`.\n- `two_connections_sharing_sub_share_budget` — same `sub` from two\n  sessions shares one bucket. Uses a subscribe + own-event recv as a\n  sync barrier to avoid the otherwise-racy \"did A's publish land\n  before B's\" ordering.\n- `distinct_subs_have_isolated_budgets` — distinct `sub`s, distinct\n  buckets.\n- `rate_zero_means_no_limit_at_all` — feature-off baseline; 32\n  publishes flow, no `rate_limited`.\n- `empty_sub_falls_back_to_conn_id_when_require_sub_is_off` — two\n  sub-less connections do not share a budget.\n- `empty_sub_is_unauthorized_publish_when_require_sub_is_on` —\n  strict-mode refusal echoes `id`.\n- `rate_limited_metric_increments_on_rejection` — the new counter\n  ticks on rejection.\n\n## Docs\n\n`docs/embedding.md` gains a \"Per-source publish rate limit\" subsection\nunder §3, documenting the four env vars, the source-resolution rules,\nand the metric name.\n\n## Verification\n\n- `cargo test --all-targets` — 298 tests pass (was 291 on trunk; +7 from\n  integration, +6 from config-tests already part of this diff).\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.",
          "timestamp": "2026-05-29T00:20:37-06:00",
          "tree_id": "0667e12be632c85a479285f502c75d88a5d479db",
          "url": "https://github.com/alternet-dev/wss-mux/commit/eb636a5e7c4ef83503b8aa96e645d6600f582378"
        },
        "date": 1780036161473,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 310,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1886,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3881,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33300,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 327,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2420,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29252,
            "range": "± 1921",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 389754,
            "range": "± 31099",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5965891,
            "range": "± 467167",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 158,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 142,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33796,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 168,
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 127,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 319,
            "range": "± 11170",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 353,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1903,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 21200,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 75,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 361,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 99,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3867,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 146,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 39035,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 379083,
            "range": "± 3574",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6571,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 138,
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
          "id": "25bca3f740ff9f1a958f31b066e07ec19049c3f5",
          "message": "feat(http): SSE read endpoint at GET /events/:stream (#91)\n\nAdd an HTTP read path that streams events as Server-Sent Events. Sibling\nof `POST /events` (produce): path-based stream selection so the URL\nreads cleanly. Optional `?key=<value>` query param narrows to one key.\n\nThe HTTP path is the consumer story for service-to-service integrations\nthat don't want a long-lived WebSocket — e.g. a separate presence_svc\nsubscribing to a `presence` stream that clients publish their heartbeat\nto. From the consumer's perspective: one curl-style GET, `text/event-\nstream` response, `data:` line per event.\n\n## Auth\n\nTwo paths, both supported simultaneously; operators expose whichever\nfits the deployment, or both:\n\n- **Shared bearer** via `WSS_MUX_READ_AUTH_TOKEN`. Constant-time\n  compare; no audience check (full read across streams). Mirrors the\n  produce-side `WSS_MUX_PUSH_AUTH_TOKEN`.\n- **JWT** via the existing handshake / OIDC validators. The token's\n  principals must intersect the stream's `subscribe` audience —\n  identical posture to WS subscribe.\n\nThe handler tries shared-bearer first, then falls back to JWT. With\nneither configured the endpoint 401s every request.\n\n## Backpressure\n\nThe SSE consumer registers a regular per-sub subscription in the\nregistry, so the existing dispatcher path drives it without any\nspecial-case code. The per-consumer queue depth uses the same\n`WSS_MUX_QUEUE_DEPTH` knob as WS subscribers. On overflow the\ndispatcher emits a keep-open `overflow` error on the consumer's\ncontrol channel; the SSE response stream observes that, emits a final\n`event: error\\ndata: overflow\\n\\n`, and ends.\n\n## Drop / cleanup\n\n`SseSubscription` (the response body) has a `Drop` impl that\nunregisters the subscription. When the consumer disconnects, axum\ndrops the response body — the same path covers both clean and dirty\ndisconnects with no extra plumbing.\n\n## No replay\n\n`Last-Event-ID` is not honored; a reconnecting consumer picks up new\nevents from the reconnect point forward. Documented in\n`docs/embedding.md`. A replay buffer is future work.\n\n## Config\n\nOne new env var: `WSS_MUX_READ_AUTH_TOKEN`. Defaults to unset — the\nendpoint accepts only JWTs when no shared bearer is configured. A\nwhitespace-only value is treated as unset (otherwise an operator with\na quoted-empty env-var would unintentionally admit `Bearer ` requests).\n\n## Tests\n\n### Unit (`src/config.rs`)\n\nThree new tests: default-unset, set-from-env, blank-treated-as-unset.\n\n### Integration (`tests/integration/sse_read.rs`)\n\nNine tests against a live server:\n\n- shared_bearer_consumer_receives_pushed_event — happy path\n- key_query_param_narrows_to_one_key — `?key=` filters correctly\n- jwt_admit_when_principals_intersect_subscribe_audience — JWT path\n- jwt_forbidden_when_principals_dont_intersect — JWT denied\n- no_bearer_is_401\n- wrong_shared_bearer_falls_through_to_jwt_and_401s — wrong shared\n  bearer does NOT silently admit\n- unknown_stream_is_404\n- dropping_the_response_unregisters_the_subscription — cleanup path\n- shared_bearer_takes_priority_over_jwt — both paths can coexist\n\nBackpressure overflow is not asserted end-to-end here; the per-sub\noverflow code path is the same one WS subscribers use and is already\ncovered by the dispatcher's own tests. A stress test would be the\nright place to wire in a wall-clock-driven assertion.\n\n## Docs\n\n`docs/embedding.md` gains a \"Reading events over HTTP (SSE)\" section\ndocumenting the path shape, auth matrix, replay/backpressure semantics,\nand a curl example.\n\n## Verification\n\n- `cargo test --all-targets` — 310 tests pass (was 301 on trunk; +9\n  integration tests; +3 config unit tests already part of the diff).\n- `cargo clippy --all-targets --all-features -- -D warnings` — clean.\n- `cargo fmt --all -- --check` — clean.",
          "timestamp": "2026-05-29T11:57:53-06:00",
          "tree_id": "fe65b9d49e4db7cc656ebf20c961ebea8875dd93",
          "url": "https://github.com/alternet-dev/wss-mux/commit/25bca3f740ff9f1a958f31b066e07ec19049c3f5"
        },
        "date": 1780077993483,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 247,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1617,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3629,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29140,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 323,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2435,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29165,
            "range": "± 1238",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 380948,
            "range": "± 22698",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4934700,
            "range": "± 376049",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 139,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 132,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31631,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 155,
            "range": "± 2",
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 127,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 145,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 301,
            "range": "± 3221",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 332,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1939,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 24948,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 74,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 412,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 100,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2337,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 151,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 43573,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 661,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 374875,
            "range": "± 2126",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6942,
            "range": "± 30",
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
          "id": "8806bae660f25546e192c551cf7bc7fe2e1e8d9b",
          "message": "feat(client-rust): scaffold wss-mux-client crate + subscribe (#93)\n\nAdds a new `clients/rust/` crate that ships a Rust SDK for wss-mux,\nsymmetric in shape with the TypeScript SDK in `clients/typescript/`.\nSubscribe lands in this PR; publish follows in a separate PR; both\nSDKs publish on the same v0.6.0 tag.\n\nThe crate is standalone — no dependency on `wss-mux` the server crate.\nThe handful of wire types it needs are duplicated under\n`src/types.rs` so the SDK's dependency graph stays minimal (`tokio`,\n`tokio-tungstenite`, `serde`, `serde_json`, `thiserror`,\n`futures-util`).\n\n## Surface\n\n```rust\nlet client = WssMuxClient::builder()\n    .url(\"wss://realtime.example.com/stream\")\n    .get_token(|| async { Ok(token_provider.token().await?) })\n    .build()\n    .await?;\n\nlet mut sub = client.subscribe(\"chat_messages\", Some(\"room-42\")).await?;\nwhile let Some(event) = sub.recv().await {\n    match event {\n        Ok(ev) => println!(\"{:?}\", ev.payload),\n        Err(e) => { eprintln!(\"{e}\"); break; }\n    }\n}\n```\n\n`Subscription::recv()` yields `Result<EventFrame, WssMuxError>`. Per-sub\nfatal codes (`unauthorized_subscribe`, `unknown_stream`,\n`duplicate_subscription_id`, `overflow`) deliver one `Err(Protocol)` and\nthen close the channel. Drop on `Subscription` posts an unsubscribe\nover the wire — best-effort if the connection has already gone away.\n\n## Architecture\n\n- `WssMuxClient` is a cheap `Arc`-shared handle. It holds an\n  `mpsc::Sender<Command>` into a single drive task.\n- The drive task is the only owner of the WebSocket halves. It runs a\n  `tokio::select!` over `cmd_rx.recv()` and `ws.next()` and routes\n  events to per-subscription channels.\n- On connection close: if 4400 (`bad_frame`) we fail-fast (it's a\n  protocol bug on our side); on 4401 we refresh the token via\n  `get_token` before reconnecting; otherwise we back off and retry.\n  Active subscriptions replay on every successful reconnect.\n- The `Sec-WebSocket-Protocol: wss-mux` subprotocol is negotiated\n  internally; consumers don't see it.\n- Server pings: explicit Pong reply in the drive task's select arm\n  (tungstenite surfaces Ping to the consumer; not auto-handled).\n\n## Tests\n\n8 integration tests against an inline mock WebSocket server (\n`tests/client.rs`):\n\n- connects + sends auth frame\n- subscribe with key + without key\n- event frame delivered to matching subscription\n- per-sub fatal error frame terminates subscription\n- reconnect + replay subscriptions after server-side close\n- close code 4401 invokes `get_token` again\n- dropping a `Subscription` posts unsubscribe\n\nPlus 5 unit tests in `src/types.rs` covering frame roundtrips and\nerror-code serialization.\n\n`cargo build --all-targets`, `cargo test`, `cargo clippy --all-targets\n-- -D warnings`, `cargo fmt --all -- --check` all green from the crate\nroot.\n\n## CI\n\nNew `.github/workflows/rust-client-ci.yml` mirrors\n`typescript-ci.yml`'s pattern: separate workflow gated on\n`clients/rust/**` paths so server-side PRs don't pay the cost. Uses\n`Swatinem/rust-cache` keyed to the client workspace.\n\n## Out of scope for this PR\n\n- Publish — comes next; `Subscription`'s API surface stays unchanged.\n- Examples beyond `basic.rs`, e2e test gated on a live server, the\n  cargo-publish release step — separate PRs per the plan.",
          "timestamp": "2026-05-29T13:11:26-06:00",
          "tree_id": "e6f50e2dec43a630fae185db8aea0fbd6071cdf6",
          "url": "https://github.com/alternet-dev/wss-mux/commit/8806bae660f25546e192c551cf7bc7fe2e1e8d9b"
        },
        "date": 1780082405452,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 254,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1654,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3705,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29194,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 322,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2355,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 30380,
            "range": "± 1392",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 341049,
            "range": "± 21784",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3389686,
            "range": "± 243581",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 139,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 133,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 32026,
            "range": "± 436",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 157,
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 127,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 133,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 147,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 307,
            "range": "± 7272",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 332,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1947,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 25262,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
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
            "value": 404,
            "range": "± 1",
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
            "value": 4225,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 150,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 43901,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 664,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 378992,
            "range": "± 2323",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6950,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 134,
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
          "id": "4aad55c91db2603621a3f3fc70c2ca509962844a",
          "message": "feat(client-ts): add publish() to WssMuxClient (#92)\n\nAdds a `publish()` method on `WssMuxClient` so a single WebSocket\nconnection covers both subscribe and publish, matching the v0.6 wire\nextension that landed in #87–#90. The HTTP `POST /events` path is\nunchanged; this is the WS-side surface for clients that already keep\na connection open for subscribe (chat send, presence heartbeat,\ncollab edits — the use cases that don't fit a long-lived server-to-\nserver bearer).\n\n## Surface\n\n```ts\nawait client.publish(\"chat_messages\", \"room-42\", {\n  from: \"alice\",\n  text: \"hello\",\n});\n```\n\nOverloads: `publish(stream, payload)` and `publish(stream, key, payload)`.\n\nAck-by-absence: the server does not send a success frame. The promise\nresolves once the settle window (`publishSettleMs`, default 250 ms)\nelapses without an error frame matching the publish's id. Within that\nwindow the SDK matches incoming `error` frames by id and rejects the\nmatching promise with a `ProtocolError`. `onError` still fires for\nparity with subscribe — the awaitable rejection is the convenience\nchannel for inline callers; observers stay observable.\n\n## Pending-publish bookkeeping\n\nA `Map<PublishId, Deferred<void>>` holds in-flight publishes.\n\n- Server error frame with a matching id → reject + delete + onError.\n- Settle timer fires first → resolve + delete.\n- Connection closes (clean or dirty) → reject all with\n  `ConnectionClosedError`, then the existing close/reconnect flow runs.\n\nPublish ids are `pub-N` so the routing in `onMessage` can distinguish\nthem from subscribe-side `sub-N` ids when both kinds of error are in\nflight on the same connection.\n\n## Fix: coalesce concurrent connects from `idle`\n\nWhile building the multi-concurrent-publish test, I tripped a latent\nbug in `ensureReady()`: the state transition to `\"connecting\"` and\nthe `readyDeferred` setup both happened *after* `await getToken()`,\nso two concurrent first-time `ensureReady()` callers each saw `idle`\nand each kicked off its own `connect()` — racing WebSockets and\ndropping frames. Subscribe never tripped this because it just sets\nlocal state and waits for replay; publish has to actually `send()`\nafter connect.\n\nFix: in `connect()`, set `state = \"connecting\"` and create the\n`readyDeferred` synchronously before awaiting `getToken()`. Existing\ntests stay green.\n\n## Wire types\n\n`PublishFrame` joins the `ClientFrame` union; `ErrorCode` gains\n`unauthorized_publish` and `publish_payload_too_large` to match the\nserver's new codes. Both new types are re-exported from\n`@alternet/wss-mux-client`.\n\n## Tests\n\n`tests/publish.test.mjs` — 7 new tests against the mock WS server:\n\n- publish frame shape: id/stream/key/payload, key-omitted variant\n- server error frame rejects the awaited `publish()` with ProtocolError\n- multiple concurrent publishes settle independently (one rejects, two\n  resolve)\n- connection drop while pending → `ConnectionClosedError`\n- publish on a closed client → `ClientUsageError`\n- error frame for an unknown id still surfaces via `onError`\n\nFull TS suite: 19 pass, 1 e2e skip. `tsc --noEmit` clean.\n\n## Docs\n\n`README.md` and `examples/basic.ts` document the new method and the\n`publishSettleMs` option.",
          "timestamp": "2026-05-29T16:58:12-06:00",
          "tree_id": "7dae5da10a847215d2071064d79fbcf49f3e2b82",
          "url": "https://github.com/alternet-dev/wss-mux/commit/4aad55c91db2603621a3f3fc70c2ca509962844a"
        },
        "date": 1780096013038,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 257,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1616,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3665,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29039,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 328,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2411,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28456,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 354225,
            "range": "± 26355",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4504107,
            "range": "± 321682",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 137,
            "range": "± 572",
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
            "value": 31981,
            "range": "± 69",
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 132,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 132,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 148,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 303,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 334,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1934,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 24199,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 74,
            "range": "± 1",
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
            "value": 401,
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
            "value": 4104,
            "range": "± 28",
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
            "value": 43484,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 665,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 374694,
            "range": "± 1976",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7254,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 134,
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
          "id": "c007f53da008acf368f74513ad28473dac8046c9",
          "message": "feat(client-rust): add publish() (#94)\n\nLands `WssMuxClient::publish(stream, key, payload)` so a single\nWebSocket covers both subscribe and publish, symmetric with the TS\nSDK's `client.publish()` (PR #92).\n\n## Surface\n\n```rust\nclient\n    .publish(\"chat_messages\", Some(\"room-42\"), json!({\"text\": \"hi\"}))\n    .await?;\n```\n\nAck-by-absence: the returned future resolves once the configured\npublish settle window elapses without a matching error frame. Within\nthe window an `error` frame whose `id` matches the publish rejects\nwith `WssMuxError::Protocol { code, .. }`. Connection drops reject\nwith `WssMuxError::ConnectionClosed`.\n\n## Internals\n\n- A `pending_pubs: HashMap<PublishId, oneshot::Sender<Result>>` lives\n  on the drive task. New publishes write the frame, insert into the\n  map, and push a settle timer onto a per-connection\n  `FuturesUnordered`.\n- The drive's select loop gains a settle arm. When a timer fires, the\n  publish id is looked up; if the entry is still pending the awaiter\n  resolves. Error frames matching a pending publish id take priority\n  over the settle timer (the error path removes the entry, so when\n  the timer eventually fires it finds nothing and is a no-op).\n- On connection close (clean or dirty) the drive drains\n  `pending_pubs` and rejects each with the close reason. That\n  prevents a false-positive `Ok(())` from a stale settle timer after\n  the WS is gone.\n\n## Builder\n\nNew `.publish_settle(Duration)` setter. Defaults to `250 ms` —\nmatches the TS SDK's `publishSettleMs` default and comfortably above\ntypical wide-area RTT. Tests use `25 ms` to keep the suite fast.\n\n## Tests\n\n`tests/publish.rs` — 7 integration tests against a mock WS server:\n\n- publish sends a `publish` frame with id/stream/key/payload and resolves\n- publish without key omits the field\n- publish rejects with Protocol error on matching server error frame\n- multiple concurrent publishes settle independently\n- publish after `close()` returns `WssMuxError::Closed`\n- error frame for unknown publish id is a no-op (no global error hook\n  in the Rust SDK; orphans are dropped, connection stays up)\n- publish rejects with ConnectionClosed when the server drops the\n  connection while a publish is pending\n\nSuite total: 21 pass (5 unit + 8 subscribe + 7 publish + 1 doctest).\n`cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --\n--check` clean.\n\n## Docs\n\n`README.md` adds a `client.publish` section and a publish-settle row\nunder builder options. `examples/basic.rs` shows the publish call\nalongside subscribe.",
          "timestamp": "2026-05-29T17:47:06-06:00",
          "tree_id": "e5b534bebb12bc87c3c37eb59a8fb2d4db71bda5",
          "url": "https://github.com/alternet-dev/wss-mux/commit/c007f53da008acf368f74513ad28473dac8046c9"
        },
        "date": 1780098945527,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 298,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1903,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3998,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 33262,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 327,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2415,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28556,
            "range": "± 1774",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 387221,
            "range": "± 42981",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4340197,
            "range": "± 256515",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 158,
            "range": "± 445",
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
            "value": 33608,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 166,
            "range": "± 2",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_hit",
            "value": 112,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 130,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 147,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 329,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 356,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1912,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 21087,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 75,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 377,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 116,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2899,
            "range": "± 26",
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
            "value": 21549,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 381801,
            "range": "± 4834",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6324,
            "range": "± 18",
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
          "id": "270dc120d132b2fd712ecf279a09d8768513be44",
          "message": "feat(client-rust): SDK e2e benches against in-process wss-mux (#97)\n\nAdds `cargo bench --bench sdk_e2e` in the Rust client crate. Each\nbench spawns a real `wss-mux` server in-process on an ephemeral port\nand drives the SDK against it, so the numbers reflect the full SDK ↔\nserver roundtrip (serialize → WS write → server parse + audience +\ndispatch → WS read → SDK deserialize → channel hand-off) rather than\na mock or microbench.\n\n## Benches\n\n- `publish_self_roundtrip` — single publish + own subscribe recv;\n  ~1.2 ms on M-series loopback.\n- `publish_throughput/{10,100}` — N publishes back-to-back, then\n  receive all N. Reports events/sec via criterion's `Throughput`.\n- `subscribe_one` — bind a subscription, drop it (sends unsubscribe);\n  ~20 µs on loopback.\n\n## Dependencies\n\nDev-only — published SDK dependency graph is unaffected:\n\n- `wss-mux` (path = \"../..\") — server crate, for the in-process bind.\n- `axum` — for `axum::serve` in the bench harness.\n- `jsonwebtoken` — HS256-sign the bench's JWT directly.\n- `criterion` with `async_tokio` — bench driver.\n\n## CI\n\n`.github/workflows/perf.yml` gains a `sdk-benchmarks (report-only)`\njob mirroring the existing wss-mux microbench job. Same posture: never\nfail the build, comment alerts on PRs, persist the trend on trunk to\nthe gh-pages dashboard under `dev/bench/sdk/`. `--measurement-time 2\n--warm-up-time 1` keeps the whole job well under a minute on a hosted\nrunner.\n\n## Settle window\n\nThe client builder is configured with `publish_settle: Duration::ZERO`\nso the publish() future resolves immediately and the bench's `recv()`\ncaptures the actual server roundtrip latency. The throughput bench\nexploits this same property to publish N back-to-back before draining\nthe receiver.\n\n## Rate limiter\n\nThe bench-side `Config` zeroes `inbound_rate_per_sec`,\n`inbound_burst`, and the `ws_publish_*` knobs. The bench's whole\npurpose is to measure throughput from one principal hammering one\nstream — the limiter behavior is its own bench (`per_source_rate_limit`\nin the server crate).",
          "timestamp": "2026-05-29T19:36:16-06:00",
          "tree_id": "0385db0507740f7fe4e51f3fa5a58715c9e78335",
          "url": "https://github.com/alternet-dev/wss-mux/commit/270dc120d132b2fd712ecf279a09d8768513be44"
        },
        "date": 1780105497621,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 255,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1617,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3654,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29331,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 323,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2478,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29456,
            "range": "± 1731",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 323002,
            "range": "± 19052",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4607634,
            "range": "± 268402",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 145,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 138,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 32272,
            "range": "± 114",
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 126,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 127,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 302,
            "range": "± 10169",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 333,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1933,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 23528,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 348,
            "range": "± 2",
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
            "value": 4112,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 149,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 43961,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 660,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 376315,
            "range": "± 3928",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6881,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 135,
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
          "id": "ec62eb580268d0f38dc4ade198b31596f683a0c4",
          "message": "feat(e2e): cross-SDK harness + Rust e2e + TS e2e expansion (#96)\n\nAdds a repo-level e2e harness that boots a real `wss-mux` binary,\nmints a JWT, and runs both SDK e2e suites against it. Cross-SDK wire\ncompatibility now has an end-to-end gate, not just per-SDK unit\ntests.\n\n## Harness\n\n`tests/e2e/`:\n\n- `manifest.yaml` — minimal three-stream manifest (`chat_messages`\n  for the happy path, `presence` for wildcard, `readonly` for\n  default-deny).\n- `mint_token.sh` — HS256-signed JWT minter using only openssl + bash\n  (no jq / python dependency).\n- `run.sh` — orchestration. Builds the release binary, picks a fixed\n  ephemeral port, boots the server with a per-run secret +\n  `WSS_MUX_HANDSHAKE_SIGNING_KEY` / `WSS_MUX_PUSH_AUTH_TOKEN` /\n  `WSS_MUX_READ_AUTH_TOKEN`, waits for `/health`, then runs each\n  SDK's gated e2e suite. Trap cleans up the server on any exit.\n- `README.md` — what's covered, env-var contract, requirements.\n\n```bash\ntests/e2e/run.sh         # rust + typescript\ntests/e2e/run.sh rust    # rust only\ntests/e2e/run.sh ts      # typescript only\n```\n\nLocal: ~5 s including server boot. CI: ~30 s on a cold runner.\n\n## Rust e2e (`clients/rust/tests/e2e.rs`)\n\n4 gated tests, skip if `WSS_MUX_E2E_URL` is unset so `cargo test` on\ntrunk doesn't try to dial a missing server:\n\n- connect + subscribe + WS publish own-event roundtrip\n- publish to readonly stream → `Protocol(unauthorized_publish)`\n- HTTP `POST /events` push observed by WS subscriber (cross-\n  transport)\n- unsubscribe stops event delivery\n\nThe HTTP-push helper hand-rolls a tiny tokio-based POST so the SDK\ncrate stays reqwest-free.\n\n## TS e2e (`clients/typescript/tests/e2e.test.mjs`)\n\nReplaces the single-subscribe scaffold with 4 gated tests\nsymmetric to the Rust suite, plus a cross-client publish→subscribe\ntest (one SDK instance publishes, another instance on the same\nserver observes).\n\n## CI (`.github/workflows/sdk-e2e.yml`)\n\nPath-gated to wire schema (`src/envelope.rs`, `src/manifest.rs`),\nserver dispatch (`src/server/**`), auth (`src/auth.rs`), either SDK\n(`clients/**`), the harness itself, or the workflow file. Other PRs\nskip the job — per-PR CI stays fast while wire-compat regressions\nare pinned whenever the surface moves.\n\n## Docs\n\n`tests/e2e/README.md` documents the harness end-to-end. Both SDK\nREADMEs gain a one-line pointer to it under Development.",
          "timestamp": "2026-05-29T22:29:11-06:00",
          "tree_id": "55a32026244353a1fba47cc5e6c20e56325325c1",
          "url": "https://github.com/alternet-dev/wss-mux/commit/ec62eb580268d0f38dc4ade198b31596f683a0c4"
        },
        "date": 1780115815805,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 135,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1010,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 2003,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 18603,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 232,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 1911,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 23806,
            "range": "± 999",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 271479,
            "range": "± 22214",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3884130,
            "range": "± 574108",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 100,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 72,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 24919,
            "range": "± 747",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_hit",
            "value": 59,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 83,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 86,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 85,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 92,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 200,
            "range": "± 3071",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 462,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1786,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 16065,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 42,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 273,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2178,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 28926,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 345,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 297858,
            "range": "± 7142",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 2892,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 87,
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
          "id": "8010420f3c8a763326322e231f017b9fef777ec3",
          "message": "ci(release): add cargo-publish job for wss-mux-client (#98)\n\nMirrors the npm-publish job's structure so the v0.6.0 tag ships the\nRust SDK to crates.io alongside @alternet/wss-mux-client to npm and\nthe binaries / Homebrew formula to their existing channels.\n\n## Posture\n\nIdempotent: queries `https://crates.io/api/v1/crates/wss-mux-client/$VERSION`\nbefore publishing. 200 ⇒ already on the registry, skip cleanly. 404 ⇒\nnew version (or first-ever publish if the crate exists), proceed.\nSame retag-friendly behavior as the npm-publish + github-release\njobs — a partial release can be re-driven without manually skipping\nalready-completed steps.\n\n## Validation\n\nTwo pre-flight checks:\n\n1. `CARGO_REGISTRY_TOKEN` secret must be set. If missing, fail fast\n   with the link to crates.io's token page rather than letting\n   `cargo publish` produce a less-actionable \"unauthorized\" error.\n2. `Cargo.toml` version must match the git tag (stripped of the `v`\n   prefix). Catches a bumped tag with a stale `Cargo.toml`, which\n   would otherwise publish the wrong version.\n\nBoth checks fail the workflow before any registry call.\n\n## Auth\n\nToken-based — crates.io's Trusted Publishing is not GA as of\nmid-2026, so the OIDC pattern used by `npm-publish` doesn't apply\nhere. `CARGO_REGISTRY_TOKEN` is a publish-scoped API token from\ncrates.io (Account → Settings → New Token), stored as a repo\nsecret. The workflow injects it only into the publish step; no\nother step can read it.\n\n## Maintainer action items before tagging v0.6.0\n\n- `CARGO_REGISTRY_TOKEN` repo secret created (one-time).\n- `wss-mux-client` name claimed on crates.io (one-time `cargo\n  publish` of the current version, or an org claim).\n\nBoth are no-ops for subsequent releases.",
          "timestamp": "2026-05-29T22:42:31-06:00",
          "tree_id": "966b009f3bb1b9d6bc4f7b80979ce6bafae90310",
          "url": "https://github.com/alternet-dev/wss-mux/commit/8010420f3c8a763326322e231f017b9fef777ec3"
        },
        "date": 1780116668361,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 244,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1467,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3215,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 26344,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 346,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 3101,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 35409,
            "range": "± 1183",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 496430,
            "range": "± 37767",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 6122761,
            "range": "± 221286",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 148,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 122,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 35798,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 145,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_hit",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 116,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 119,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 133,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 295,
            "range": "± 1392",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 638,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1901,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 22002,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 410,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 4301,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 40827,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 684,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 412038,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 5944,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 119,
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
          "id": "ccb5f427f5f5ca007f1b65340a1cad2a5c825c47",
          "message": "docs: v0.6 sweep — WS publish, SSE read, SDKs, presence_svc pattern (#100)\n\nBrings the three repo-level docs in line with the v0.6 wire surface\nthat landed across #87–#94 and #95.\n\n## architecture.md\n\n- Dispatch diagram now shows the SSE read endpoint (HTTP server) and\n  publish on the WS server, matching `build_app`'s route table.\n- New \"Unified dispatch\" callout: HTTP push, WS publish, WS subscribe,\n  and SSE read all share the dispatcher and registry — the four wire\n  surfaces differ only in framing.\n- HTTP server section gains a paragraph on `GET /events/:stream`\n  (auth matrix + dispatcher-side parity with WS subscribe).\n- WS server section now covers the `publish` frame: audience check,\n  per-stream payload cap, per-source rate limit, dispatch via the\n  same path as HTTP `POST /events`. Notes that publish rejections\n  are keep-open (same posture as `overflow`).\n\n## embedding.md\n\n- New \"Client SDKs\" section pointing to the TS and Rust SDKs under\n  `clients/` and noting they're wire-compatible.\n- New \"Building a presence service on top\" section — the canonical\n  end-to-end example the v0.6 design was shaped around. Shows the\n  manifest stanza, a browser-side heartbeat using the TS SDK, and an\n  HTTP SSE consumer pattern for a backend presence_svc. Closes with\n  the schematic of clients ↔ wss-mux ↔ presence_svc.\n\n## README.md\n\n- Quick start's Subscribe example switches from a raw WebSocket\n  snippet to the TS SDK quickstart — more representative of how\n  consumers actually integrate. Same example also demonstrates\n  `client.publish()` over the same connection.\n- Adds pointers to the Rust SDK and the SSE read endpoint for the\n  non-WS consumer story.\n\nNo changes to `docs/protocol.md` — wire surface for publish frames\nand `unauthorized_publish` / `publish_payload_too_large` was already\ndocumented when the frames landed.",
          "timestamp": "2026-05-29T22:58:04-06:00",
          "tree_id": "73f0fc3ad7213ae0210fc9cf6f278c6788b84fb2",
          "url": "https://github.com/alternet-dev/wss-mux/commit/ccb5f427f5f5ca007f1b65340a1cad2a5c825c47"
        },
        "date": 1780117568069,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 230,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1508,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3125,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 25831,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 251,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 1827,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 22388,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 313639,
            "range": "± 29701",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4823792,
            "range": "± 320046",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 124,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 112,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 26072,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 130,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_hit",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 97,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 113,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 246,
            "range": "± 5960",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 278,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1486,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 16613,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 262,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2952,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 111,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 31096,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 570,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 305998,
            "range": "± 3644",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 5207,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 108,
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
          "id": "db7eada2577339c8f58d21ed6539cdceabb39738",
          "message": "chore: bump version to 0.6.0 (#101)\n\nCuts v0.6.0, the WS-publish + SDK release. Three crates / packages\nbump in lockstep:\n\n- `wss-mux` (server) `0.5.3` → `0.6.0`\n- `wss-mux-client` (Rust SDK) `0.5.3` → `0.6.0` — first crates.io publish\n- `@alternet/wss-mux-client` (TS SDK) `0.5.3` → `0.6.0`\n\n## What ships in v0.6.0\n\nWire surface (server):\n\n- `publish` ClientFrame variant (#87, #88) — a WS client emits an\n  event over the connection it already holds open. Same downstream\n  dispatch as HTTP `POST /events`.\n- Per-stream `publish` audience in the manifest (#87) — default-deny,\n  parallel to the existing `subscribe` field. Field rename from\n  `audience` → `subscribe` is the wire-breaking part (pre-1.0\n  manifest schema bump).\n- `unauthorized_publish` + `publish_payload_too_large` error codes\n  (#88), keep-open.\n- `PerSourceRateLimiter` keyed by JWT `sub` (#89, #90) — per-source\n  publish budget, sweeper task, configurable via\n  `WSS_MUX_WS_PUBLISH_RATE` / `_BURST` / `_REQUIRE_SUB` /\n  `_IDLE_TTL_SECS`.\n- `GET /events/:stream` SSE read endpoint (#91) — the backend-\n  consumer story; auth supports shared-bearer\n  (`WSS_MUX_READ_AUTH_TOKEN`) and JWT.\n\nClient SDKs:\n\n- TypeScript SDK gains `client.publish()` (#92), `publishSettleMs`\n  builder option, `unauthorized_publish` / `publish_payload_too_large`\n  in the `ErrorCode` union.\n- Rust SDK (`clients/rust/`) lands as a new crate (#93, #94) —\n  subscribe + publish, reconnect + token-refresh + heartbeat\n  internal, typed errors. `connect_async`-based, minimal deps.\n- Cross-SDK e2e harness (#96) — single-shell entrypoint that boots\n  the server, mints a JWT, runs both SDKs' gated e2e suites; new\n  path-gated `sdk-e2e.yml` workflow.\n- Rust SDK criterion benches against an in-process server (#97):\n  publish_self_roundtrip, publish_throughput, subscribe_one.\n\nCI / release:\n\n- `cargo-publish` job in `release.yml` (#98) — idempotent via\n  crates.io HEAD check, validates Cargo.toml vs tag, uses\n  `CARGO_REGISTRY_TOKEN`. TP migration tracked separately in #99.\n\nDocs:\n\n- `docs/architecture.md` + `docs/embedding.md` swept for the new\n  surface (#100), including the presence_svc pattern.\n\n## Maintainer action items before the tag fires\n\n- `CARGO_REGISTRY_TOKEN` secret set (token scoped to\n  `wss-mux-client`, `publish-new` + `publish-update`).\n- crates.io has space for the `wss-mux-client` name (first publish\n  will claim it).",
          "timestamp": "2026-05-29T22:58:09-06:00",
          "tree_id": "957add102e32fd1ebea536699cceee102c0e7277",
          "url": "https://github.com/alternet-dev/wss-mux/commit/db7eada2577339c8f58d21ed6539cdceabb39738"
        },
        "date": 1780117610670,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 258,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1803,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3656,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29664,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 326,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2422,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29287,
            "range": "± 1684",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 335591,
            "range": "± 19071",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3541563,
            "range": "± 281366",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 139,
            "range": "± 420",
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
            "value": 32199,
            "range": "± 82",
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
            "name": "per_source_try_take_hit",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 132,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 307,
            "range": "± 3532",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 332,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1932,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 24955,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 404,
            "range": "± 2",
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
            "value": 3013,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 38476,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 664,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 381300,
            "range": "± 2857",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7337,
            "range": "± 59",
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
          "id": "46374939844e2750bcb0eeddab6dfc6797f2585c",
          "message": "fix(oidc): split discovery_url from issuer (closes #86) (#104)\n\nLets the public `iss` value tokens carry diverge from the URL\nwss-mux fetches discovery from. The Keycloak\n`KC_HOSTNAME_BACKCHANNEL_DYNAMIC` pattern (and analogous setups for\nother IdPs) deliberately splits these — tokens minted on either the\npublic or in-cluster host claim the public issuer, while metadata\nendpoints serve via whichever host the request actually arrived on.\n\n## Before\n\n`OidcConfig.issuer` did two jobs at once: token-iss validation\n(`set_issuer(&[issuer])` in auth) AND the discovery base\n(`<issuer>/.well-known/openid-configuration` in oidc). Operators\nrunning Keycloak BACKCHANNEL_DYNAMIC had to point\n`WSS_MUX_OIDC_ISSUER` at the public URL (token-iss matches) and then\npin `WSS_MUX_OIDC_JWKS_URL` to the in-cluster endpoint to skip\ndiscovery entirely — sidestepping OIDC's \"the IdP tells you the\nendpoints\" guarantee.\n\n## After\n\nNew optional `OidcConfig.discovery_url` (env\n`WSS_MUX_OIDC_DISCOVERY_URL`). When set, it's the discovery base;\nwhen unset, the discovery base is `issuer` (existing behavior).\nToken-iss validation is unchanged — still pinned to `issuer`.\n`jwks_url` is also unchanged; if set it still short-circuits the\nwhole discovery dance.\n\n```bash\nWSS_MUX_OIDC_ISSUER=https://auth.example.com/realms/x       # token iss\nWSS_MUX_OIDC_DISCOVERY_URL=http://keycloak:8080/realms/x    # discovery base\n# jwks_url left unset — the discovery doc tells us where keys live,\n# and BACKCHANNEL_DYNAMIC makes that URL in-cluster too.\n```\n\n## Tests\n\n- `src/oidc.rs`: `discovery_url_base_overrides_issuer` —\n  resolve_jwks_url fetches discovery at the in-cluster URL and\n  returns the in-cluster jwks_uri the doc carries.\n- `src/oidc.rs`: `jwks_url_still_short_circuits_discovery_url` —\n  explicit jwks_url still wins over discovery_url.\n- `src/config.rs`: `oidc_discovery_url_is_read_independently_of_issuer`\n  — env-var plumbing.\n\nPlus the existing OIDC test fixtures + the integration test's\n`oidc_cfg` helper threaded for the new field.\n\n## Out of scope\n\n- Caching changes (discovery cache is already separate from JWKS).\n- Validation strictness (`set_issuer(&[issuer])` is correct as-is).\n- Multi-issuer (still one IdP per multiplexer).\n\n## Docs\n\n- `docs/embedding.md` OIDC section: new env var + a paragraph on\n  the BACKCHANNEL_DYNAMIC use case.\n- `README.md` config table: new row, JWKS row clarified to reference\n  the discovery base.",
          "timestamp": "2026-05-30T00:42:44-06:00",
          "tree_id": "3a151ed0d74fda485428b16b2e04ce66a009d6cb",
          "url": "https://github.com/alternet-dev/wss-mux/commit/46374939844e2750bcb0eeddab6dfc6797f2585c"
        },
        "date": 1780123834811,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 236,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1592,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3072,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 27129,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 256,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 1818,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 22165,
            "range": "± 1203",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 284759,
            "range": "± 35504",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3805778,
            "range": "± 55307",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 122,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 26000,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_hit",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 97,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 116,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 244,
            "range": "± 3422",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 274,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1480,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 16571,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 265,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3065,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 110,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 31112,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 570,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 302422,
            "range": "± 1790",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 4896,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 109,
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
          "id": "a34f57c765fade212775697a53532f3741ca3510",
          "message": "feat(cli): wss-mux validate-manifest subcommand (#102) (#105)\n\nAdds `wss-mux validate-manifest <path>`: parses + validates a\nstreams manifest using upstream's own parser, then exits 0 with a\none-line summary or 1 with the validation error. No env vars\nrequired; no server runtime spun up.\n\n## Why\n\nEmbedders that codegen the streams manifest (e.g. application-\ntemplate's Python AST walker) need to drift-check the generated YAML\nagainst upstream's schema. Today they re-implement enough of\n`src/manifest.rs` to read the file, which means the v0.6 `audience`\n→ `subscribe` rename + new `publish` field broke them silently — and\nevery future schema evolution carries the same hazard.\n\nThis subcommand makes upstream's parser the source of truth: the\nembedder's CI pre-flight shells out instead of carrying its own copy\nof the schema.\n\n## Surface\n\n```\n$ wss-mux validate-manifest config/wss-mux/streams.yaml\nok: 2 streams loaded (notification_banner, suspension_status_card)\n$ echo $?\n0\n\n$ wss-mux validate-manifest broken.yaml\nerror: ...\n$ echo $?\n1\n\n$ wss-mux validate-manifest\nusage: wss-mux validate-manifest <path>\n$ echo $?\n2\n```\n\n## Dispatch\n\n`main()` now inspects `argv[1]` synchronously before any tokio\nruntime spin-up:\n\n- `validate-manifest` → run synchronously, exit.\n- `help` / `--help` / `-h` → print usage, exit 0.\n- anything else (or no arg) → fall through to `server_main()`, which\n  carries the existing async server behavior verbatim. The new\n  surface is **additive** — `wss-mux` with no args still starts the\n  server unchanged.\n\nThe dispatch lives ahead of `Config::from_env()`, so embedders don't\nneed to satisfy the server's env-var contract when they're just\nvalidating a file.\n\n## Tests\n\n`tests/integration/validate_manifest.rs` spawns the built binary via\n`env!(\"CARGO_BIN_EXE_wss-mux\")` and asserts:\n\n- valid YAML → exit 0, stdout includes the stream count + names\n- invalid YAML → non-zero exit, stderr mentions \"error\"\n- missing path → non-zero exit\n- missing positional arg → non-zero exit with a usage hint\n\nTiny in-test tempdir helper rather than a new dev-dep.\n\n## Out of scope\n\n- `--json` structured output. Marked \"maybe\" in #102; if an embedder\n  needs structured diagnostics we can add it as a follow-up. Today's\n  callers grep stdout / treat exit code as the signal.\n- Wire-protocol validation, audience-set validation against an\n  external IdP, etc. — separate concerns per the issue.\n\n## Docs\n\n`README.md` gains a one-paragraph note under the operational\nsection.",
          "timestamp": "2026-05-30T01:32:04-06:00",
          "tree_id": "a2bdd2bdc8955cbc1a90c4dbc2f49dd3c54c4dd8",
          "url": "https://github.com/alternet-dev/wss-mux/commit/a34f57c765fade212775697a53532f3741ca3510"
        },
        "date": 1780126844696,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 251,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1650,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3704,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 28895,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 325,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2445,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29157,
            "range": "± 1723",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 366966,
            "range": "± 23579",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5154928,
            "range": "± 274398",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 139,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 132,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 32020,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 158,
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 147,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 334,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 333,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 2089,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 23636,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 349,
            "range": "± 2",
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
            "value": 3829,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 21624,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 663,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 374416,
            "range": "± 4282",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6917,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 135,
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
          "id": "ebd4de160f381268afd7ed16933812812c5fc1af",
          "message": "docs(client-ts): pin getToken contract; assert fresh token on wire after 4401 (closes #103) (#106)\n\nTwo pieces:\n\n## Doc the contract\n\nThe TS SDK's `getToken` callback has a precise contract the README\nonly gestured at: called on initial connect AND on close-code 4401,\n*and the freshly returned value reaches the wire* (no stale-cache\nwindow across a 4401-driven reconnect). Other reconnects reuse the\ncached token so a flapping connection doesn't hammer the issuer.\n\nEmbedders rely on this for token-rotation safety nets — a SPA's\nshort-lived JWT can refresh out-of-band, and the SDK auto-recovers\non the next 4401 without re-mounting the client.\n\nREADME's getToken section now spells out:\n\n- Exactly when the SDK invokes it (initial connect + 4401 reconnect).\n- That the fresh value reaches the wire on the post-4401 connection,\n  not a cached previous value.\n- The non-4401 reconnect caching behavior, so consumers don't expect\n  refresh on every disconnect.\n- Pre-emptive refresh is out of scope (app-layer concern).\n\n## Strengthen the test\n\n`reconnect.test.mjs` already had \"4401 close triggers fresh\ngetToken() call\", which asserts *getToken was called twice* and the\ntwo returned tokens differ — but not that the second one actually\nmakes it onto the wire. That's a real failure mode (cache-\ninvalidation order bug, stale closure capture, etc.) and not\nsomething the existing test would catch.\n\nNew test `4401 reconnect sends the freshly-fetched token on the\nwire`: the mock server records auth frames per connection; the\ntest asserts the post-4401 connection's auth-frame `token` is\nexactly the new value returned by getToken, not the initial one.\n\nSuite total: 20 pass / 4 skip (e2e), was 19 / 4. typecheck clean.\n\n## Out of scope\n\n- Pre-emptive refresh strategy. Per the issue, that's an app-layer\n  feature — the SDK only handles the 4401-driven safety net.\n- Refresh contracts for the Rust SDK. Same shape but different\n  language idioms; a follow-up if asked.",
          "timestamp": "2026-05-30T01:37:03-06:00",
          "tree_id": "64eb35311c85b49c4f029b1211a6f1e4886a727d",
          "url": "https://github.com/alternet-dev/wss-mux/commit/ebd4de160f381268afd7ed16933812812c5fc1af"
        },
        "date": 1780127144537,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 257,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1614,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3739,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29323,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 321,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2382,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29171,
            "range": "± 1772",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 357649,
            "range": "± 23236",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3573365,
            "range": "± 122278",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 137,
            "range": "± 372",
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
            "value": 31684,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 156,
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 130,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 133,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 309,
            "range": "± 8474",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 340,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 2082,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 25192,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 350,
            "range": "± 1",
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
            "value": 3871,
            "range": "± 20",
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
            "value": 43297,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 660,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 379385,
            "range": "± 3388",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6952,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 135,
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
          "id": "4211a5be187d4c41110e4f1b186bb12a7a9d645c",
          "message": "chore: bump version to 0.6.1 (#107)\n\nPatch release on top of 0.6.0. Three packages move in lockstep:\n\n- `wss-mux` (server) 0.6.0 → 0.6.1\n- `wss-mux-client` (Rust SDK) 0.6.0 → 0.6.1\n- `@alternet/wss-mux-client` (TS SDK) 0.6.0 → 0.6.1\n\n## What ships\n\n- OIDC: split discovery_url from issuer (#104) — new\n  `WSS_MUX_OIDC_DISCOVERY_URL` env var lets the public `iss` value\n  tokens carry diverge from the URL wss-mux fetches discovery from.\n  Resolves the Keycloak `KC_HOSTNAME_BACKCHANNEL_DYNAMIC` pain point\n  where the only workaround was pinning `WSS_MUX_OIDC_JWKS_URL` and\n  giving up the rest of the discovery doc.\n- `wss-mux validate-manifest <path>` subcommand (#105) — parses +\n  validates a streams manifest using upstream's own parser. Exits 0\n  with a one-line summary, 1 on validation error, 2 on usage error.\n  No env vars required, no server runtime. For embedders' CI\n  pre-flight on codegen'd manifests.\n- TS SDK: getToken contract pinned in the README + a wire-level test\n  asserting the freshly-fetched token actually reaches the auth\n  frame on a 4401-driven reconnect (#106). No behavior change in\n  the SDK; characterization test locks the invariant against\n  regression.\n\nThe Rust SDK has no source changes; the version bump is the\nlockstep half. crates.io publishes a 0.6.1 with identical code to\n0.6.0 so the trio's versions stay aligned.\n\n## No wire-breaking changes\n\nPatch release — all wire surfaces (envelope, manifest schema, error\ncodes, close codes, subprotocol) are byte-identical to 0.6.0. A\n0.6.0 client speaks to a 0.6.1 server and vice versa, no changes.",
          "timestamp": "2026-05-30T01:50:52-06:00",
          "tree_id": "dc31810eff8881947a81cf1fdbe87e0b364401c6",
          "url": "https://github.com/alternet-dev/wss-mux/commit/4211a5be187d4c41110e4f1b186bb12a7a9d645c"
        },
        "date": 1780127969795,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 251,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1616,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3811,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 29412,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 331,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2383,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 29241,
            "range": "± 1732",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 352669,
            "range": "± 25511",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 3589943,
            "range": "± 201299",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 141,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 136,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 31986,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 159,
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
            "name": "per_source_try_take_hit",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 129,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 146,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 313,
            "range": "± 1484",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 332,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1936,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 23701,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
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
            "value": 334,
            "range": "± 3",
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
            "value": 3945,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 149,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 42057,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 659,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 370232,
            "range": "± 2671",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 7816,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 134,
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
          "id": "05eb2fe212c39065159ef5eb538c194dd27c876a",
          "message": "docs(embedding): surface validate-manifest in §3 with CI wiring example (#110)\n\nPer #108. The subcommand from #105 was documented in README but never\nmentioned in `docs/embedding.md` — the canonical embedding guide. An\nembedder reading the guide end-to-end never sees it, which is exactly\nthe discoverability gap that prompted the issue (the reporter shipped\na codegen pipeline to v0.6.1 specifically to consume #104 and didn't\nrealize #105 had also landed).\n\nAdds a \"Pre-flight validation\" subsection right after the manifest\nexample in §3, showing the two-step pattern: drift check on the\ncodegen output (the embedder's existing tooling) plus\n`wss-mux validate-manifest` against the on-disk file (upstream's\nparser as the schema source of truth). Calls out the exit codes so\nCI consumers can wire the pass/fail signal straight.\n\nNo code change.",
          "timestamp": "2026-05-30T18:31:40-06:00",
          "tree_id": "0360264840bdce48d4a7fb6016a149f1d15ca808",
          "url": "https://github.com/alternet-dev/wss-mux/commit/05eb2fe212c39065159ef5eb538c194dd27c876a"
        },
        "date": 1780188018451,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 259,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1622,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3846,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 28708,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 327,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2454,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28667,
            "range": "± 1080",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 361103,
            "range": "± 33518",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 4625987,
            "range": "± 315508",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 138,
            "range": "± 407",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 165,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 39738,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 194,
            "range": "± 4",
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
            "name": "per_source_try_take_hit",
            "value": 86,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 129,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 145,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 316,
            "range": "± 3645",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 332,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1938,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 23390,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 328,
            "range": "± 0",
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
            "value": 3909,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 21597,
            "range": "± 10726",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 662,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 378397,
            "range": "± 3509",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 8255,
            "range": "± 86",
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
          "id": "9981e9cbd4d5f51864991e8daac07bd4c9f69aae",
          "message": "feat(client-rust): expose ConnectionState via watch::Receiver (#111)\n\nPer #109. The TS SDK ships `onStateChange(state)` so consumers can\ndrive UI affordances (a \"reconnecting…\" indicator, a publish-\ndisabled guard until `Ready`) off the connection lifecycle. The\nRust SDK shipped without an equivalent — consumers could only infer\nstate from operation outcomes, which makes a Tauri-style desktop\nshell unable to surface `Reconnecting` mid-blip or\n`Authenticating` during the auth-frame pre-ack window.\n\n## Surface\n\n```rust\nlet mut rx = client.state_changes();           // watch::Receiver\nwhile rx.changed().await.is_ok() {\n    let s = *rx.borrow();\n    // drive UI…\n}\n\n// Or sample without subscribing:\nlet s = client.state();                        // Copy snapshot\n```\n\n`ConnectionState` is a `Debug, Clone, Copy, Eq, Hash` enum with\nvariants matching the TS SDK's `onStateChange` vocabulary:\n`Idle, Connecting, Authenticating, Ready, Reconnecting, Closing,\nClosed`.\n\n## Why watch and not broadcast / callback\n\nPer the issue's stated preference (option b). `watch` is idiomatic\nasync Rust:\n\n- Composes natively with `tokio::select!` — Tauri IPC bridges use\n  it directly.\n- No `Send + 'static` closure ceremony.\n- The \"what's the current state right now\" sampling is free\n  (`borrow()`).\n\nThe cost is coalescing on rapid transitions: when the drive crosses\ntwo phases between two consumer polls (the `Authenticating` window\non fast loopback is sub-millisecond), the receiver reads the later\nstate when it wakes. This is the right shape for \"is it Ready or\nnot\" UI logic; it's documented in the README + on `state_changes`\nso consumers who need every step have informed expectations.\n\n## Internals\n\n- `Drive` gains a `watch::Sender<ConnectionState>` field, plus a\n  small `set_state` helper.\n- Transition sites: `connect()` brackets the auth write with\n  `Connecting → Authenticating → Ready`; the reconnect path emits\n  `Reconnecting` before backoff; the `Close` command emits\n  `Closing` before sending the close frame; terminal outcomes\n  (Closed, BadFrame, ReconnectExhausted, initial-connect-fail)\n  emit `Closed`.\n- `ClientInner` stores a `watch::Receiver<ConnectionState>` for\n  `state()` snapshots and `state_changes()` clones.\n\n## Tests\n\n`clients/rust/tests/connection_state.rs` — 4 new:\n\n- `build_lands_client_in_ready_state` — initial-connect happy path.\n- `close_transitions_to_closed` — explicit shutdown lands Closed.\n- `state_changes_yields_a_transition_on_close` — `changed().await`\n  fires; the borrowed value after the change is `Closed`.\n- `server_close_drives_reconnecting_then_back_to_ready` — transient\n  abnormal close walks through `Reconnecting` and lands back in\n  `Ready`. The \"must pass through Reconnecting\" assertion holds\n  here because the backoff sleep guarantees the drive parks long\n  enough for the receiver to observe it.\n\nSuite total: 30 pass (was 26). clippy + fmt clean.\n\n## Docs\n\n- `clients/rust/README.md` documents `state()`, `state_changes()`,\n  the enum vocabulary, and the watch-coalescing trade-off.\n- Module-level rustdoc on `ConnectionState` spells out the happy-\n  path + reconnect + close sequences.",
          "timestamp": "2026-05-30T18:40:54-06:00",
          "tree_id": "b1170b56c4f59fdc0a5efafa188f413d446f177a",
          "url": "https://github.com/alternet-dev/wss-mux/commit/9981e9cbd4d5f51864991e8daac07bd4c9f69aae"
        },
        "date": 1780188570483,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 241,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1452,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3284,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 26480,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 354,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 3121,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 35648,
            "range": "± 1503",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 511420,
            "range": "± 33889",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 7180309,
            "range": "± 492953",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 149,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 122,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 36308,
            "range": "± 2309",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 144,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/shallow",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_pluck/deep",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_hit",
            "value": 86,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 117,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 121,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 316,
            "range": "± 1595",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 640,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1891,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 20587,
            "range": "± 973",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 64,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 404,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 83,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 3174,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 42558,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 684,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 417962,
            "range": "± 741",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 5950,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 121,
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
          "id": "331e88ede82d9b812862381b03203f37c65dfcfa",
          "message": "chore: bump version to 0.6.2 (#112)\n\nPatch release on top of 0.6.1. Three packages move in lockstep:\n\n- `wss-mux` (server) 0.6.1 → 0.6.2\n- `wss-mux-client` (Rust SDK) 0.6.1 → 0.6.2\n- `@alternet/wss-mux-client` (TS SDK) 0.6.1 → 0.6.2\n\n## What ships\n\n- Rust SDK: `ConnectionState` observability via `state()` snapshot\n  and `state_changes()` watch::Receiver (#111). Mirrors the TS SDK's\n  `onStateChange` callback for cross-SDK parity — embedders writing\n  transport adapters that target both runtimes (e.g. a Tauri shell\n  wrapping the Rust SDK and proxying to a React webview) can surface\n  the same `Reconnecting` / `Authenticating` UI affordances on both\n  sides.\n- Embedding docs: validate-manifest CI wiring example surfaced in\n  §3 of `docs/embedding.md` (#110). Closes the discoverability gap\n  flagged by an embedder who shipped the upgrade for #104 and\n  didn't notice #105 had also landed the subcommand they would have\n  used.\n\nThe server has no source changes; the version bump is the lockstep\nhalf. The TS SDK has no source changes either.\n\n## No wire-breaking changes\n\nPatch release — all wire surfaces (envelope, manifest schema, error\ncodes, close codes, subprotocol) are byte-identical to 0.6.1. A\n0.6.1 client speaks to a 0.6.2 server and vice versa, no changes.",
          "timestamp": "2026-05-30T18:54:50-06:00",
          "tree_id": "cc5ea0576036f32d87a152cc7592fbe255eab0ed",
          "url": "https://github.com/alternet-dev/wss-mux/commit/331e88ede82d9b812862381b03203f37c65dfcfa"
        },
        "date": 1780189408501,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 319,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1890,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3886,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 32845,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 335,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2436,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28513,
            "range": "± 1818",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 387672,
            "range": "± 31082",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5304885,
            "range": "± 386200",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 158,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 138,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33821,
            "range": "± 290",
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
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_hit",
            "value": 88,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 125,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 127,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 129,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 147,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 337,
            "range": "± 5605",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 354,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1909,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 21150,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 76,
            "range": "± 0",
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
            "value": 368,
            "range": "± 18",
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
            "value": 3815,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 146,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 39033,
            "range": "± 1065",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 387133,
            "range": "± 1931",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6886,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 136,
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
          "id": "726aeacb09810fa94a820b156c1a7e55c49210d7",
          "message": "release: emit a clean-auditing Homebrew formula (#113)\n\n`brew test-bot --only-tap-syntax` on `alternet-dev/homebrew-tap` was\nflagging three things on the wss-mux formula we render. Fix all\nthree at the source so the next release pushes a clean file:\n\n1. **Redundant `version` line.** When the asset URL embeds the version\n   (`/v0.6.2/wss-mux-v0.6.2-…`), Homebrew infers it. The explicit\n   `version \"${VERSION}\"` line was no-op redundancy that\n   `brew audit` rejects. Drop it.\n\n2. **Non-standard SPDX license.** Homebrew wants SPDX *expressions*\n   via the `any_of:` form, not a quoted SPDX-string with `OR`. Switch\n   `license \"MIT OR Apache-2.0\"` → `license any_of: [\"MIT\", \"Apache-2.0\"]`.\n\n3. **Missing URL for Intel macOS.** We deliberately don't build an\n   Intel-Mac bottle, but `brew readall --os=all --arch=all` evaluates\n   the formula for every (OS, arch) combo and crashed with \"formula\n   requires at least a URL\" for (macos, x86_64). Two changes together\n   keep readall happy while still leaving the formula effectively\n   Apple-Silicon-only on macOS:\n\n   - add `depends_on arch: :arm64` inside `on_macos`, which aborts an\n     actual install on Intel macOS before any URL is fetched, with a\n     clear architecture-mismatch error;\n   - mirror the arm64 url/sha256 into an `on_intel` block as a stub.\n     `brew style` rejects `url` directly inside `on_macos` (only the\n     inner arch/version blocks are allowed), and that URL is never\n     actually downloaded thanks to the depends_on above — it exists\n     only to satisfy readall's per-(OS, arch) URL requirement.\n\nThe `version.to_s` interpolation in the `test` stanza keeps working\nbecause Homebrew populates `version` from the parsed URL.",
          "timestamp": "2026-06-01T11:31:47-06:00",
          "tree_id": "a4e80b02830e9cc5255ec9f513cb82edb2355dab",
          "url": "https://github.com/alternet-dev/wss-mux/commit/726aeacb09810fa94a820b156c1a7e55c49210d7"
        },
        "date": 1780335641901,
        "tool": "cargo",
        "benches": [
          {
            "name": "cbor/encode_event_frame",
            "value": 318,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_event_frame",
            "value": 1989,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/encode_relay_batch_32",
            "value": 3948,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cbor/decode_relay_batch_32",
            "value": 34103,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1",
            "value": 331,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10",
            "value": 2383,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/100",
            "value": 28727,
            "range": "± 1800",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/1000",
            "value": 393455,
            "range": "± 32828",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_fanout/10000",
            "value": 5994040,
            "range": "± 1133931",
            "unit": "ns/iter"
          },
          {
            "name": "dispatch_no_subscribers",
            "value": 157,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_small",
            "value": 142,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/default_large",
            "value": 33762,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "envelope_from_value/nested_paths",
            "value": 169,
            "range": "± 2",
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
            "name": "per_source_try_take_hit",
            "value": 87,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/100",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/1000",
            "value": 129,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_mixed/sources/10000",
            "value": 147,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_try_take_cold_insert",
            "value": 312,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/100",
            "value": 355,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/1000",
            "value": 1902,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "per_source_sweep_idle/sources/10000",
            "value": 21424,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1",
            "value": 75,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10",
            "value": 351,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10",
            "value": 100,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/100",
            "value": 2320,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/100",
            "value": 147,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/1000",
            "value": 21338,
            "range": "± 451",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/1000",
            "value": 738,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/unkeyed/10000",
            "value": 380745,
            "range": "± 2754",
            "unit": "ns/iter"
          },
          {
            "name": "registry_matches/keyed/10000",
            "value": 6343,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "registry_subscribe_unsubscribe",
            "value": 136,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}