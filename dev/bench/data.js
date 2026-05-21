window.BENCHMARK_DATA = {
  "lastUpdate": 1779388622672,
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
      }
    ]
  }
}