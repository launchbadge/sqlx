use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlx::postgres::protocol::{Bind, DataRow, RowDescription};
use sqlx::postgres::protocol::{Decode, Encode};

fn bench(c: &mut Criterion) {
    c.bench_function("decode_data_row", |b| {
        b.iter(|| {
            let _ = DataRow::decode(&black_box(b"\0\x03\0\0\0\x011\0\0\0\x012\0\0\0\x013")[..]);
        });
    });

    c.bench_function( "decode_row_description",|b| {
        b.iter(|| {
            let _ = RowDescription::decode(&black_box(b"\0\x02user_id\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0number_of_pages\0\0\0\0\0\0\0\0\0\x05\0\0\0\0\0\0\0\0\0")[..]);
        });
    });

    c.bench_function("encode_bind", |b| {
        let mut buf = Vec::new();

        b.iter(|| {
            black_box(Bind {
                portal: "__sqlx_portal_5121",
                statement: "__sqlx_statement_5121",
                formats: &[1],
                values_len: 2,
                values: &[(-1_i8) as _, 0, 0, 0, 1, 0, 0, 0, 25],
                result_formats: &[1],
            })
            .encode(&mut buf);

            buf.clear();
        });
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
