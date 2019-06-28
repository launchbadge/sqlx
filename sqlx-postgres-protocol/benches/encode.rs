#[macro_use]
extern crate criterion;

use criterion::Criterion;
use sqlx_postgres_protocol::{Encode, PasswordMessage, Response, Severity, StartupMessage};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("encode Response::builder()", |b| {
        let mut dst = Vec::new();
        b.iter(|| {
            dst.clear();
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

    c.bench_function("encode PasswordMessage::cleartext", |b| {
        let mut dst = Vec::new();
        b.iter(|| {
            dst.clear();
            PasswordMessage::cleartext("8e323AMF9YSE9zftFnuhQcvhz7Vf342W4cWU")
                .encode(&mut dst)
                .unwrap();
        })
    });

    c.bench_function("encode StartupMessage", |b| {
        let mut dst = Vec::new();
        b.iter(|| {
            dst.clear();
            StartupMessage::builder()
                .param("user", "postgres")
                .param("database", "postgres")
                .param("DateStyle", "ISO, MDY")
                .param("IntervalStyle", "iso_8601")
                .param("TimeZone", "UTC")
                .param("extra_float_digits", "3")
                .param("client_encoding", "UTF-8")
                .build()
                .encode(&mut dst)
                .unwrap();
        })
    });

    c.bench_function("encode Password(MD5)", |b| {
        let mut dst = Vec::new();
        b.iter(|| {
            dst.clear();
            PasswordMessage::md5(
                "8e323AMF9YSE9zftFnuhQcvhz7Vf342W4cWU",
                "postgres",
                &[10, 41, 20, 150],
            )
            .encode(&mut dst)
            .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
