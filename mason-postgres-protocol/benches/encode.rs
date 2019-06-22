#[macro_use]
extern crate criterion;

use criterion::Criterion;
use mason_postgres_protocol::{Encode, Response, Severity};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("encode Response(Builder)", |b| {
        let mut dst = Vec::new();
        b.iter(|| {
            dst.truncate(0);
            Response::builder()
                .severity(Severity::Notice)
                .code("42710")
                .message("extension \"uuid-ossp\" already exists, skipping")
                .file("extension.c")
                .line(1656)
                .routine("CreateExtension")
                .encode(&mut dst)
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
