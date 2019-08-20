#![feature(async_await)]

#[macro_use]
extern crate criterion;

use bytes::BytesMut;
use criterion::{BatchSize, Criterion};
use sqlx::pg::protocol::Message;

const MESSAGE_DATA_ROW: &[u8] = b"D\0\0\0\x19\0\x02\0\0\0\x08\0\0\0\0\0\0\0\x05\0\0\0\x03whaD\0\0\0\xd6\0\x02\0\0\0\x08\0\0\0\0\0\0\0\x07\0\0\0\xc0Spicy jalapeno bacon ipsum dolor amet doner venison ground round burgdoggen salami fatback jerky sirloin t-bone beef. Ribeye chuck tenderloin pastrami short loin capicola beef tri-tip venison.";
const MESSAGE_COMMAND_COMPLETE: &[u8] = b"C\0\0\0\rSELECT 4\0";
const MESSAGE_READY_FOR_QUERY: &[u8] = b"Z\0\0\0\x05I";
const MESSAGE_RESPONSE: &[u8] = b"N\0\0\0rSNOTICE\0VNOTICE\0C42P07\0Mrelation \"tasks\" already exists, skipping\0Fparse_utilcmd.c\0L206\0RtransformCreateStmt\0\0";
const MESSAGE_BACKEND_KEY_DATA: &[u8] = b"K\0\0\0\x0c\0\0!\x9a\x853\x89\xf5";
const MESSAGE_PARAMETER_STATUS: &[u8] = b"S\0\0\01server_version\011.4 (Debian 11.4-1.pgdg90+1)\0";

fn bench(c: &mut Criterion, name: &'static str, input: &'static [u8]) {
    c.bench_function(name, move |b| {
        let mut buf = BytesMut::new();

        for _ in 0..1000 {
            buf.extend_from_slice(input);
        }

        b.iter_batched(
            || buf.clone(),
            |mut buf| {
                while let Some(_body) = Message::decode(&mut buf).unwrap() {}
                assert!(buf.is_empty());
            },
            BatchSize::LargeInput,
        );
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench(
        c,
        "postgres - decode - Message - DataRow (x 1000)",
        MESSAGE_DATA_ROW,
    );
    bench(
        c,
        "postgres - decode - Message - ReadyForQuery (x 1000)",
        MESSAGE_READY_FOR_QUERY,
    );
    bench(
        c,
        "postgres - decode - Message - CommandComplete (x 1000)",
        MESSAGE_COMMAND_COMPLETE,
    );
    bench(
        c,
        "postgres - decode - Message - Response (x 1000)",
        MESSAGE_RESPONSE,
    );
    bench(
        c,
        "postgres - decode - Message - BackendKeyData (x 1000)",
        MESSAGE_BACKEND_KEY_DATA,
    );
    bench(
        c,
        "postgres - decode - Message - ParameterStatus (x 1000)",
        MESSAGE_PARAMETER_STATUS,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
