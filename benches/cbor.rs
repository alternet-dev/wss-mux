//! Microbenchmarks for the CBOR codec on the relay + binary-client
//! hot path (`ServerFrame::Event`, `RelayBatch`).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

use wss_mux::envelope::{EventEnvelope, ServerFrame};
use wss_mux::server::relay::RelayBatch;

fn event_frame() -> ServerFrame {
    ServerFrame::Event {
        id: "s1".into(),
        stream: "chat_messages".into(),
        key: Some("room-42".into()),
        payload: json!({"from": "alice", "text": "hello benchmark"}),
    }
}

fn relay_batch(n: usize) -> RelayBatch {
    RelayBatch {
        events: (0..n)
            .map(|i| EventEnvelope {
                stream: "chat_messages".into(),
                key: Some(format!("room-{i}")),
                payload: json!({"n": i, "text": "hello benchmark"}),
            })
            .collect(),
    }
}

fn bench_cbor(c: &mut Criterion) {
    let frame = event_frame();
    let mut frame_buf = Vec::new();
    ciborium::into_writer(&frame, &mut frame_buf).expect("encode event frame");

    let batch = relay_batch(32);
    let mut batch_buf = Vec::new();
    ciborium::into_writer(&batch, &mut batch_buf).expect("encode relay batch");

    let mut group = c.benchmark_group("cbor");
    group.bench_function("encode_event_frame", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            ciborium::into_writer(black_box(&frame), &mut buf).unwrap();
            buf
        });
    });
    group.bench_function("decode_event_frame", |b| {
        b.iter(|| ciborium::from_reader::<ServerFrame, _>(black_box(&frame_buf[..])).unwrap());
    });
    group.bench_function("encode_relay_batch_32", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            ciborium::into_writer(black_box(&batch), &mut buf).unwrap();
            buf
        });
    });
    group.bench_function("decode_relay_batch_32", |b| {
        b.iter(|| ciborium::from_reader::<RelayBatch, _>(black_box(&batch_buf[..])).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_cbor);
criterion_main!(benches);
