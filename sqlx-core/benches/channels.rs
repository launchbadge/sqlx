use criterion::{Bencher, BenchmarkId, Criterion, criterion_group, criterion_main, Throughput};

fn bench_spsc(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_spsc(threaded, count, capacity)");

    for threaded in [false, true] {
        for count in [100u64, 1000, 10_000] {
            group.throughput(Throughput::Bytes(size_of::<u64>() as u64 * count));

            for capacity in [16usize, 64, 256] {
                group.bench_with_input(
                    BenchmarkId::from_parameter(
                        format!("tokio::sync::mpsc({threaded}, {count}, {capacity})")
                    ),
                    &(threaded, count, capacity),
                    bench_spsc_tokio,
                );

                group.bench_with_input(
                    BenchmarkId::from_parameter(
                        format!("flume({threaded}, {count}, {capacity})")
                    ),
                    &(threaded, count, capacity),
                    bench_spsc_flume,
                );

                group.bench_with_input(
                    BenchmarkId::from_parameter(
                        format!("double_buffer({threaded}, {count}, {capacity})")
                    ),
                    &(threaded, count, capacity),
                    bench_spsc_double_buffer,
                );
            }
        }
    }

    group.finish();
}

fn bench_spsc_tokio(bencher: &mut Bencher, &(threaded, count, capacity): &(bool, u64, usize)) {
    bencher.to_async(build_spsc_runtime(threaded)).iter(|| async {
        let (mut tx, mut rx) = tokio::sync::mpsc::channel(capacity);

        tokio::try_join!(
            tokio::spawn(async move {
                for i in 0 .. count {
                    tx.send(i).await.expect("BUG: channel closed early");
                }
            }),
            tokio::spawn(async move {
                for expected in 0 .. count {
                    assert_eq!(rx.recv().await, Some(expected));
                }

                assert_eq!(rx.recv().await, None);
            })
        ).unwrap();
    });
}

fn bench_spsc_flume(bencher: &mut Bencher, &(threaded, count, capacity): &(bool, u64, usize)) {
    bencher.to_async(build_spsc_runtime(threaded)).iter(|| async {
        let (mut tx, mut rx) = flume::bounded(capacity);

        tokio::try_join!(
            tokio::spawn(async move {
                for i in 0 .. count {
                    tx.send_async(i).await.expect("BUG: channel closed early");
                }
            }),
            tokio::spawn(async move {
                for expected in 0 .. count {
                    assert_eq!(rx.recv_async().await, Ok(expected));
                }

                assert_eq!(rx.recv_async().await.ok(), None);
            })
        ).unwrap();
    });
}

fn bench_spsc_double_buffer(bencher: &mut Bencher, &(threaded, count, capacity): &(bool, u64, usize)) {
    bencher.to_async(build_spsc_runtime(threaded)).iter(|| async {
        let (mut tx, mut rx) = sqlx_core::common::channel::double_buffer::channel(capacity);

        tokio::try_join!(
            tokio::spawn(async move {
                for i in 0 .. count {
                    tx.send(i).await.expect("BUG: channel closed early");
                }
            }),
            tokio::spawn(async move {
                for expected in 0 .. count {
                    assert_eq!(rx.recv().await, Some(expected));
                }

                assert_eq!(rx.recv().await, None);
            })
        ).unwrap();
    });
}

fn build_spsc_runtime(threaded: bool) -> tokio::runtime::Runtime {
    let mut builder = if threaded {
        let mut builder = tokio::runtime::Builder::new_multi_thread();
        builder.worker_threads(2);
        builder
    } else {
        tokio::runtime::Builder::new_current_thread()
    };

    builder
        .enable_all()
        .build()
        .unwrap()
}

criterion_group!(benches, bench_spsc);
criterion_main!(benches);
