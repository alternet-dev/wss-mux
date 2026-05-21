//! Microbenchmarks for envelope parsing (`EventEnvelope::from_value`,
//! `pluck`) — called once per inbound producer event.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

use wss_mux::envelope::{pluck, EnvelopePaths, EventEnvelope};

fn bench_from_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("envelope_from_value");

    let default_paths = EnvelopePaths {
        stream: "stream",
        key: "key",
        payload: "payload",
    };
    let small = json!({"stream": "chat", "key": "room-1", "payload": {"text": "hi"}});
    group.bench_function("default_small", |b| {
        b.iter(|| EventEnvelope::from_value(black_box(&small), default_paths));
    });

    let large_payload = serde_json::Value::Array(
        (0..200)
            .map(|i| json!({"i": i, "v": "xxxxxxxxxxxxxxxx"}))
            .collect(),
    );
    let large = json!({"stream": "chat", "key": "room-1", "payload": large_payload});
    group.bench_function("default_large", |b| {
        b.iter(|| EventEnvelope::from_value(black_box(&large), default_paths));
    });

    let nested_paths = EnvelopePaths {
        stream: "meta.topic",
        key: "meta.room",
        payload: "data",
    };
    let nested = json!({"meta": {"topic": "chat", "room": "42"}, "data": {"text": "hi"}});
    group.bench_function("nested_paths", |b| {
        b.iter(|| EventEnvelope::from_value(black_box(&nested), nested_paths));
    });

    group.finish();
}

fn bench_pluck(c: &mut Criterion) {
    let mut group = c.benchmark_group("envelope_pluck");
    let v = json!({"a": {"b": {"c": {"d": "deep"}}}, "shallow": "x"});
    group.bench_function("shallow", |b| {
        b.iter(|| pluck(black_box(&v), black_box("shallow")));
    });
    group.bench_function("deep", |b| {
        b.iter(|| pluck(black_box(&v), black_box("a.b.c.d")));
    });
    group.finish();
}

criterion_group!(benches, bench_from_value, bench_pluck);
criterion_main!(benches);
