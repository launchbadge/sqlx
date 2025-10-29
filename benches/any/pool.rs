use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use sqlx_core::any::AnyPoolOptions;
use std::fmt::{Display, Formatter};
use tracing::Instrument;

#[derive(Debug)]
struct Input {
    threads: usize,
    tasks: usize,
    pool_size: usize,
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "threads: {}, tasks: {}, pool size: {}",
            self.threads, self.tasks, self.pool_size
        )
    }
}

fn bench_pool(c: &mut Criterion) {
    sqlx::any::install_default_drivers();
    tracing_subscriber::fmt::try_init().ok();

    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let inputs = [
        Input {
            threads: 1,
            tasks: 2,
            pool_size: 20,
        },
        Input {
            threads: 2,
            tasks: 4,
            pool_size: 20,
        },
        Input {
            threads: 4,
            tasks: 8,
            pool_size: 20,
        },
        Input {
            threads: 8,
            tasks: 16,
            pool_size: 20,
        },
        Input {
            threads: 16,
            tasks: 32,
            pool_size: 64,
        },
        Input {
            threads: 16,
            tasks: 128,
            pool_size: 64,
        },
    ];

    let mut group = c.benchmark_group("Bench Pool");

    for input in inputs {
        group.bench_with_input(BenchmarkId::from_parameter(&input), &input, |b, i| {
            bench_pool_with(b, i, &database_url)
        });
    }

    group.finish();
}

fn bench_pool_with(b: &mut Bencher, input: &Input, database_url: &str) {
    let _span = tracing::info_span!(
        "bench_pool_with",
        threads = input.threads,
        tasks = input.tasks,
        pool_size = input.pool_size
    )
    .entered();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(input.threads)
        .build()
        .unwrap();

    let pool = runtime.block_on(async {
        AnyPoolOptions::new()
            .min_connections(input.pool_size)
            .max_connections(input.pool_size)
            .test_before_acquire(false)
            .connect(database_url)
            .await
            .expect("error connecting to pool")
    });

    for num in 1..=input.tasks {
        let pool = pool.clone();

        runtime.spawn(
            async move { while pool.acquire().await.is_ok() {} }
                .instrument(tracing::info_span!("task", num)),
        );
    }

    b.to_async(&runtime).iter(|| {
        async {
            if let Err(e) = pool.acquire().await {
                panic!("failed to acquire connection: {e:?}");
            }
        }
        .instrument(tracing::info_span!("iter"))
    });

    drop(pool.close());
}

criterion_group!(benches, bench_pool,);
criterion_main!(benches);
