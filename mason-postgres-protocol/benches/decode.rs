#[macro_use]
extern crate criterion;

use bytes::Bytes;
use criterion::{black_box, Criterion};
use mason_postgres_protocol::{Decode, Response};

fn criterion_benchmark(c: &mut Criterion) {
    // NOTE: This is sans header (for direct decoding)
    const NOTICE_RESPONSE: &[u8]  = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    c.bench_function("decode Response", |b| {
        b.iter(|| {
            let _ = Response::decode(black_box(Bytes::from_static(NOTICE_RESPONSE))).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
