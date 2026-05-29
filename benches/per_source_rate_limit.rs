//! Microbenchmarks for `PerSourceRateLimiter` — the hot path on every
//! WS publish frame when the per-source rate limit is configured.
//!
//! The interesting axes are:
//!
//! - **Steady state, single source** — the cache-hit path: one lookup
//!   in DashMap + one Mutex lock + a take. This is what every
//!   subsequent frame from an established connection pays.
//! - **Steady state, many sources** — a uniform mix across N
//!   established sources. Picks up any DashMap sharding contention
//!   that the single-source bench can't.
//! - **Sweep cost vs map size** — how expensive the periodic idle
//!   eviction is as the active-source set grows.

use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use wss_mux::ratelimit::PerSourceRateLimiter;

fn limiter(idle_ttl_secs: u64) -> PerSourceRateLimiter {
    // Capacity well above any single bench iteration so we measure the
    // admit-path, not the refill-when-empty path. `refill_per_sec` is
    // generous for the same reason.
    PerSourceRateLimiter::new(1_000_000, 1_000_000, Duration::from_secs(idle_ttl_secs))
}

fn bench_steady_state_single_source(c: &mut Criterion) {
    let l = limiter(300);
    // Warm the source so we measure the hit path, not the cold insert.
    let _ = l.try_take("alice", Instant::now());

    c.bench_function("per_source_try_take_hit", |b| {
        b.iter(|| {
            let now = Instant::now();
            black_box(l.try_take(black_box("alice"), now));
        });
    });
}

fn bench_steady_state_mixed_sources(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_source_try_take_mixed");
    for n in [10usize, 100, 1000, 10_000] {
        let l = limiter(300);
        // Pre-seed all N sources so every iteration is a hit.
        let now = Instant::now();
        for i in 0..n {
            l.try_take(&format!("source-{i}"), now);
        }

        group.bench_with_input(BenchmarkId::new("sources", n), &n, |b, &n| {
            let mut i: usize = 0;
            b.iter(|| {
                let key = format!("source-{}", i % n);
                let now = Instant::now();
                black_box(l.try_take(black_box(&key), now));
                i = i.wrapping_add(1);
            });
        });
    }
    group.finish();
}

fn bench_cold_insert(c: &mut Criterion) {
    // The first-take path: lookup miss → vacant entry → fresh bucket.
    // Each iteration uses a fresh key so we never hit the warm path.
    c.bench_function("per_source_try_take_cold_insert", |b| {
        let l = limiter(300);
        let mut i: u64 = 0;
        b.iter(|| {
            let key = format!("cold-{i}");
            let now = Instant::now();
            black_box(l.try_take(black_box(&key), now));
            i = i.wrapping_add(1);
        });
    });
}

fn bench_sweep_idle(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_source_sweep_idle");
    for n in [100usize, 1000, 10_000] {
        // A pool with all-fresh entries; the sweep at `now == last_seen`
        // touches every entry but evicts none. This is the realistic
        // "nothing to evict" baseline (the eviction loop runs often, the
        // map is rarely mostly-stale).
        group.bench_with_input(BenchmarkId::new("sources", n), &n, |b, &n| {
            let l = limiter(300);
            let now = Instant::now();
            for i in 0..n {
                l.try_take(&format!("s-{i}"), now);
            }
            b.iter(|| {
                black_box(l.sweep_idle(black_box(now)));
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_steady_state_single_source,
    bench_steady_state_mixed_sources,
    bench_cold_insert,
    bench_sweep_idle,
);
criterion_main!(benches);
