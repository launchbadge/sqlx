use sqlx::Postgres;
use sqlx_test::test_type;

test_type!(bool(
    Postgres,
    bool,
    "false::boolean" == false,
    "true::boolean" == true
));

test_type!(i16(Postgres, i16, "821::smallint" == 821_i16));
test_type!(i32(Postgres, i32, "94101::int" == 94101_i32));
test_type!(i64(Postgres, i64, "9358295312::bigint" == 9358295312_i64));

test_type!(f32(Postgres, f32, "9419.122::real" == 9419.122_f32));
test_type!(f64(
    Postgres,
    f64,
    "939399419.1225182::double precision" == 939399419.1225182_f64
));

test_type!(string(
    Postgres,
    String,
    "'this is foo'" == "this is foo",
    "''" == ""
));

// TODO: BYTEA
// TODO: UUID
// TODO: CHRONO

// #[cfg_attr(feature = "runtime-async-std", async_std::test)]
// #[cfg_attr(feature = "runtime-tokio", tokio::test)]
// async fn postgres_bytes() -> anyhow::Result<()> {
//     let mut conn = connect().await?;
//
//     let value = b"Hello, World";
//
//     let rec: (bool, Vec<u8>) = sqlx::query("SELECT E'\\\\x48656c6c6f2c20576f726c64' = $1, $1")
//         .bind(&value[..])
//         .map(|row: PgRow| Ok((row.get(0)?, row.get(1)?)))
//         .fetch_one(&mut conn)
//         .await?;
//
//     assert!(rec.0);
//     assert_eq!(&value[..], &*rec.1);
//
//     Ok(())
// }
