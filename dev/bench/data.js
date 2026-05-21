window.BENCHMARK_DATA = {
  "lastUpdate": 1779348738104,
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
      }
    ]
  }
}