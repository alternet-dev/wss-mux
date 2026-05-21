window.BENCHMARK_DATA = {
  "lastUpdate": 1779383016311,
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
      }
    ]
  }
}