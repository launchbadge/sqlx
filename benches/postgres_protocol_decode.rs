#![feature(async_await)]

#[macro_use]
extern crate criterion;

use bytes::BytesMut;
use criterion::{BatchSize, Criterion};
use sqlx::pg::protocol::Message;

const MESSAGE_DATA_ROW_SMALL: &[u8] =
    b"D\0\0\0\x1a\0\x02\0\0\0\x08\0\0\0\0\0\0\0\x08\0\0\0\x04Task";
const MESSAGE_DATA_ROW_MEDIUM: &[u8] = b"D\0\0\0\x19\0\x02\0\0\0\x08\0\0\0\0\0\0\0\x05\0\0\0\x03whaD\0\0\0\xd6\0\x02\0\0\0\x08\0\0\0\0\0\0\0\x07\0\0\0\xc0Spicy jalapeno bacon ipsum dolor amet doner venison ground round burgdoggen salami fatback jerky sirloin t-bone beef. Ribeye chuck tenderloin pastrami short loin capicola beef tri-tip venison.";

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("[postgres] [decode] [Message] DataRow (small x1)", |b| {
        let buf = BytesMut::from(MESSAGE_DATA_ROW_SMALL);

        b.iter_batched(
            || buf.clone(),
            |mut buf| {
                while let Some(_body) = Message::decode(&mut buf).unwrap() {}
            },
            BatchSize::LargeInput,
        );
    });

    c.bench_function("[postgres] [decode] [Message] DataRow (small x10)", |b| {
        let mut buf = BytesMut::new();

        for _ in 0..10 {
            buf.extend_from_slice(MESSAGE_DATA_ROW_SMALL);
        }

        b.iter_batched(
            || buf.clone(),
            |mut buf| {
                while let Some(_body) = Message::decode(&mut buf).unwrap() {}
            },
            BatchSize::LargeInput,
        );
    });

    c.bench_function(
        "[postgres] [decode] [Message] DataRow (small x10_000)",
        |b| {
            let mut buf = BytesMut::new();

            for _ in 0..10_000 {
                buf.extend_from_slice(MESSAGE_DATA_ROW_SMALL);
            }

            b.iter_batched(
                || buf.clone(),
                |mut buf| {
                    while let Some(_body) = Message::decode(&mut buf).unwrap() {}
                },
                BatchSize::LargeInput,
            );
        },
    );

    c.bench_function("[postgres] [decode] [Message] DataRow (medium x1)", |b| {
        let buf = BytesMut::from(MESSAGE_DATA_ROW_MEDIUM);

        b.iter_batched(
            || buf.clone(),
            |mut buf| {
                while let Some(_body) = Message::decode(&mut buf).unwrap() {}
            },
            BatchSize::LargeInput,
        );
    });

    c.bench_function("[postgres] [decode] [Message] DataRow (medium x10)", |b| {
        let mut buf = BytesMut::new();

        for _ in 0..10 {
            buf.extend_from_slice(MESSAGE_DATA_ROW_MEDIUM);
        }

        b.iter_batched(
            || buf.clone(),
            |mut buf| {
                while let Some(_body) = Message::decode(&mut buf).unwrap() {}
            },
            BatchSize::LargeInput,
        );
    });

    c.bench_function(
        "[postgres] [decode] [Message] DataRow (medium x10_000)",
        |b| {
            let mut buf = BytesMut::new();

            for _ in 0..10_000 {
                buf.extend_from_slice(MESSAGE_DATA_ROW_MEDIUM);
            }

            b.iter_batched(
                || buf.clone(),
                |mut buf| {
                    while let Some(_body) = Message::decode(&mut buf).unwrap() {}
                },
                BatchSize::LargeInput,
            );
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
