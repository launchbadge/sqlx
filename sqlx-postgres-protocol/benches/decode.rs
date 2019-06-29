#[macro_use]
extern crate criterion;

use bytes::Bytes;
use criterion::{black_box, Criterion};
use sqlx_postgres_protocol::{BackendKeyData, Decode, ParameterStatus, Response};

fn criterion_benchmark(c: &mut Criterion) {
    const NOTICE_RESPONSE: &[u8]  = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";
    const PARAM_STATUS: &[u8] = b"session_authorization\0postgres\0";
    const BACKEND_KEY_DATA: &[u8] = b"\0\0'\xc6\x89R\xc5+";

    c.bench_function("decode Response", |b| {
        b.iter(|| {
            let _ = Response::decode(black_box(Bytes::from_static(NOTICE_RESPONSE))).unwrap();
        })
    });

    c.bench_function("decode BackendKeyData", |b| {
        b.iter(|| {
            let _ =
                BackendKeyData::decode(black_box(Bytes::from_static(BACKEND_KEY_DATA))).unwrap();
        })
    });

    c.bench_function("decode ParameterStatus", |b| {
        b.iter(|| {
            let _ = ParameterStatus::decode(black_box(Bytes::from_static(PARAM_STATUS))).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
