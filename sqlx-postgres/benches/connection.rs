#![feature(async_await)]

#[macro_use]
extern crate criterion;

use bytes::Bytes;
use criterion::{black_box, Criterion};
use sqlx_core::ConnectOptions;
use sqlx_postgres::Connection;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Connection::establish", |b| {
        let options = ConnectOptions::new()
            .port(5433) // mock
            .user("postgres")
            .database("postgres");

        b.iter(|| {
            runtime::raw::enter(runtime::native::Native, async move {
                let _conn = Connection::establish(options).await.unwrap();
            });
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
