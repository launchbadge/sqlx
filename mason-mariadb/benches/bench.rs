#[macro_use]
extern crate criterion;

use criterion::Criterion;
use mason_core::ConnectOptions;
use mason_mariadb::connection::Connection;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("establish connection", |b| {
        b.iter(|| {
            Connection::establish(ConnectOptions {
                host: "127.0.0.1",
                port: 3306,
                user: Some("root"),
                database: None,
                password: None,
            })
            .await.unwarp();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
