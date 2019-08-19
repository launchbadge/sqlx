#![feature(async_await)]

#[macro_use]
extern crate criterion;

use criterion::Criterion;
use futures::stream::TryStreamExt;
use tokio::runtime::Runtime;

use sqlx::Query as _;

const DATABASE_URL: &str = "postgres://postgres@127.0.0.1:5432/sqlx__dev";

async fn sqlx_select(conn: &sqlx::Connection<sqlx::pg::Pg>) {
    let _rows: Vec<String> = sqlx::query::<sqlx::pg::PgQuery>("SELECT name FROM contacts").fetch(conn).try_collect().await.unwrap();
}

fn rust_postgres_select(cl: &mut rust_postgres::Client) {
    let _rows: Vec<String> = cl.query("SELECT name FROM contacts", &[]).unwrap().into_iter().map(|row| {
        row.get(0)
    }).collect();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sqlx select", |b| {
        let rt = Runtime::new().unwrap();
        let conn = rt.block_on(async {
            sqlx::Connection::<sqlx::pg::Pg>::establish(DATABASE_URL).await.unwrap()
        });

        b.iter(|| {
            rt.block_on(sqlx_select(&conn));
        });
    });

    c.bench_function("rust-postgres select", |b| {
        let mut cl = rust_postgres::Client::connect(DATABASE_URL, rust_postgres::NoTls).unwrap();

        b.iter(|| {
            rust_postgres_select(&mut cl);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
