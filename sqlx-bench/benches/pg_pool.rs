use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use sqlx::PgPool;

use sqlx::postgres::PgPoolOptions;
use std::time::{Duration, Instant};

fn bench_pgpool_acquire(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_pgpool_acquire");

    for &concurrent in [5u32, 10, 50, 100, 500, 1000, 5000 /*, 10_000, 50_000*/].iter() {
        for &fair in [false, true].iter() {
            let fairness = if fair { "(fair)" } else { "(unfair)" };

            group.bench_with_input(
                format!("{concurrent} concurrent {fairness}"),
                &(concurrent, fair),
                |b, &(concurrent, fair)| do_bench_acquire(b, concurrent, fair),
            );
        }
    }

    group.finish();
}

fn do_bench_acquire(b: &mut Bencher, concurrent: u32, fair: bool) {
    let pool = sqlx::__rt::block_on(
        PgPoolOptions::new()
            // we don't want timeouts because we want to see how the pool degrades
            .acquire_timeout(Duration::from_secs(3600))
            // force the pool to start full
            .min_connections(50)
            .max_connections(50)
            // we're not benchmarking `ping()`
            .test_before_acquire(false)
            .__fair(fair)
            .connect(
                &dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set to run benchmarks"),
            ),
    )
    .expect("failed to open PgPool");

    for _ in 0..concurrent {
        let pool = pool.clone();
        sqlx::__rt::enter_runtime(|| {
            sqlx::__rt::spawn(async move {
                while !pool.is_closed() {
                    let conn = match pool.acquire().await {
                        Ok(conn) => conn,
                        Err(sqlx::Error::PoolClosed) => break,
                        Err(e) => panic!("failed to acquire concurrent connection: {e}"),
                    };

                    // pretend we're using the connection
                    sqlx::__rt::sleep(Duration::from_micros(500)).await;
                    drop(criterion::black_box(conn));
                }
            })
        });
    }

    b.iter_custom(|iters| {
        sqlx::__rt::block_on(async {
            // take the start time inside the future to make sure we only count once it's running
            let start = Instant::now();
            for _ in 0..iters {
                criterion::black_box(
                    pool.acquire()
                        .await
                        .expect("failed to acquire connection for benchmark"),
                );
            }
            start.elapsed()
        })
    });

    sqlx::__rt::block_on(pool.close());
}

criterion_group!(pg_pool, bench_pgpool_acquire);
criterion_main!(pg_pool);
