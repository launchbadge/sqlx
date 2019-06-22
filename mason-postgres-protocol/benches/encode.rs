#[macro_use]
extern crate criterion;

use criterion::{Criterion};
use mason_postgres_protocol::{Encode, NoticeResponse, Severity};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("encode NoticeResponse", |b| {
        b.iter(|| {
            let message = NoticeResponse::builder()
                .severity(Severity::Notice)
                .code("42710")
                .message("extension \"uuid-ossp\" already exists, skipping")
                .file("extension.c")
                .line(1656)
                .routine("CreateExtension")
                .build();

            let mut dst = Vec::with_capacity(message.size_hint());
            message.encode(&mut dst).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
