//! Microbenchmarks for the per-event fan-out hot path (`dispatch`).

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use wss_mux::dispatcher::{dispatch, EventOrigin};
use wss_mux::server::AppState;

#[path = "common/mod.rs"]
#[allow(dead_code)]
mod common;

fn bench_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_fanout");
    for n in [1usize, 10, 100, 1000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            // cap 64 is well above the single frame each dispatch puts
            // in a channel between drains, so dispatch never overflows
            // (which would mutate the registry mid-measurement).
            let (state, mut handles) = common::state_with_subs("bench_stream", n, 64, false);
            let envelope = common::event("bench_stream");
            b.iter_batched(
                || {
                    // Untimed: drain receivers + clone the envelope so
                    // the timed routine is pure `dispatch`.
                    for h in &mut handles {
                        h.drain();
                    }
                    envelope.clone()
                },
                |env| dispatch(black_box(&state), env, EventOrigin::Producer),
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();

    // Floor: the no-subscribers early-return path.
    c.bench_function("dispatch_no_subscribers", |b| {
        let state = AppState::new(common::bench_config(64));
        let envelope = common::event("unsubscribed_stream");
        b.iter_batched(
            || envelope.clone(),
            |env| dispatch(black_box(&state), env, EventOrigin::Producer),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_dispatch);
criterion_main!(benches);
