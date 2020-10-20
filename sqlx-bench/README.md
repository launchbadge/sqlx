SQLx Self-Benchmarks
====================

This Cargo project implements various benchmarks for SQLx using
[Criterion](https://crates.io/crates/criterion).

### Available Benchmarks

* Group `pg_pool`: benchmarks `sqlx::Pool` against a PostgreSQL server.
    * `DATABASE_URL` must be set (or in `.env`) pointing to a PostgreSQL server. 
    It should preferably be running on the same machine as the benchmarks to reduce latency. 
    * The `postgres` feature must be enabled for this benchmark to run.
    * Benchmarks:
        * `bench_pgpool_acquire`: benchmarks `Pool::acquire()` when many concurrent tasks are also using
        the pool, with or without the pool being fair. Concurrently to the benchmark iteration
        function calling and blocking on `Pool::acquire()`, a varying number of background tasks are
        also calling `acquire()` and holding the acquired connection for 500µs each before releasing
        it back to the pool. The pool is created with `.min_connections(50).max_connections(50)` so we shouldn't
        be measuring anything but the actual overhead of `Pool`'s bookeeping.

### Running

You must choose a runtime to execute the benchmarks on; the feature flags are the same as the `sqlx` crate:

```bash
cargo bench --features runtime-tokio-native-tls
cargo bench --features runtime-async-std-rustls
```

When complete, the benchmark results will be in `target/criterion/`. 
Open `target/criterion/report/index.html` or pick one of the benchmark subfolders and open
`report/index.html` there to view the results.

Benchmark Results
-------

If you want to share the results here, please follow the format below.

* [2020/07/01: `pg_pool` benchmark added to test pool fairness changes](results/2020-07-01-bench_pgpool_acquire/REPORT.md)
