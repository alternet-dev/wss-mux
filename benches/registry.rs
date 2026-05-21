//! Microbenchmarks for subscription lookup (`Registry::matches`) and
//! subscribe/unsubscribe churn.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use wss_mux::registry::Registry;

#[path = "common/mod.rs"]
#[allow(dead_code)]
mod common;

fn bench_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_matches");
    for n in [1usize, 10, 100, 1000, 10_000] {
        // Unkeyed subscriptions, unkeyed event — every binding matches.
        group.bench_with_input(BenchmarkId::new("unkeyed", n), &n, |b, &n| {
            let registry = common::registry_with_subs("bench_stream", n, false);
            b.iter(|| registry.matches(black_box("bench_stream"), black_box(None)));
        });
        // Keyed subscriptions, keyed event — the linear key-filter scan.
        group.bench_with_input(BenchmarkId::new("keyed", n), &n, |b, &n| {
            let registry = common::registry_with_subs("bench_stream", n, true);
            b.iter(|| registry.matches(black_box("bench_stream"), black_box(Some("key-0"))));
        });
    }
    group.finish();
}

fn bench_subscribe_unsubscribe(c: &mut Criterion) {
    // A subscribe+unsubscribe pair leaves the stream's binding vec
    // empty, so a persistent registry never grows across iterations.
    let registry = Registry::new();
    c.bench_function("registry_subscribe_unsubscribe", |b| {
        b.iter(|| {
            registry.subscribe(black_box("bench_stream"), 1, "s1".to_string(), None);
            registry.unsubscribe(black_box("bench_stream"), 1, "s1");
        });
    });
}

criterion_group!(benches, bench_matches, bench_subscribe_unsubscribe);
criterion_main!(benches);
