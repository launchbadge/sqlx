use sqlx::{postgres::PgConnection, Connection as _, Row};

async fn connect() -> anyhow::Result<PgConnection> {
    Ok(PgConnection::open(dotenv::var("DATABASE_URL")?).await?)
}

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        #[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
        async fn $name () -> anyhow::Result<()> {
            let mut conn = connect().await?;

            $(
                let row = sqlx::query(&format!("SELECT {} = $1, $1 as _1", $text))
                    .bind($value)
                    .fetch_one(&mut conn)
                    .await?;

                assert!(row.get::<bool, _>(0));
                assert!($value == row.get::<$ty, _>("_1"));
            )+

            Ok(())
        }
    }
}

test!(postgres_bool: bool: "false::boolean" == false, "true::boolean" == true);

test!(postgres_smallint: i16: "821::smallint" == 821_i16);
test!(postgres_int: i32: "94101::int" == 94101_i32);
test!(postgres_bigint: i64: "9358295312::bigint" == 9358295312_i64);

test!(postgres_real: f32: "9419.122::real" == 9419.122_f32);
test!(postgres_double: f64: "939399419.1225182::double precision" == 939399419.1225182_f64);

test!(postgres_text: String: "'this is foo'" == "this is foo", "''" == "");

test!(postgres_int_vec: Vec<i32>: "ARRAY[1, 2, 3]::int[]" == vec![1, 2, 3i32], "ARRAY[3, 292, 15, 2, 3]::int[]" == vec![3, 292, 15, 2, 3], "ARRAY[7, 6, 5, 4, 3, 2, 1]::int[]" == vec![7, 6, 5, 4, 3, 2, 1], "ARRAY[]::int[]" == vec![] as Vec<i32>);
test!(postgres_string_vec: Vec<String>: "ARRAY['Hello', 'world', 'friend']::text[]" == vec!["Hello", "world", "friend"]);
test!(postgres_bool_vec: Vec<bool>: "ARRAY[true, true, false, true]::bool[]" == vec![true, true, false, true]);
test!(postgres_real_vec: Vec<f32>: "ARRAY[0.0, 1.0, 3.14, 1.234, -0.002, 100000.0]::real[]" == vec![0.0, 1.0, 3.14, 1.234, -0.002, 100000.0_f32]);
test!(postgres_double_vec: Vec<f64>: "ARRAY[0.0, 1.0, 3.14, 1.234, -0.002, 100000.0]::double precision[]" == vec![0.0, 1.0, 3.14, 1.234, -0.002, 100000.0_f64]);

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn postgres_bytes() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = b"Hello, World";

    let row = sqlx::query("SELECT E'\\\\x48656c6c6f2c20576f726c64' = $1, $1")
        .bind(&value[..])
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));

    let output: Vec<u8> = row.get(1);

    assert_eq!(&value[..], &*output);

    Ok(())
}
