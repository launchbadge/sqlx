#[macro_use]
extern crate criterion;

use criterion::{Criterion, black_box};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("establish connection", |b| {
        b.iter(|| {
            let conn = Connection::establish(ConnectOptions {
                host: "127.0.0.1",
                port: 3306,
                user: Some("root"),
                database: None,
                password: None,
            });
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
