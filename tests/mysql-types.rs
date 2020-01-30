use sqlx::{mysql::MySqlConnection, Connection, Row};

async fn connect() -> anyhow::Result<MySqlConnection> {
    Ok(MySqlConnection::open(dotenv::var("DATABASE_URL")?).await?)
}

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        #[cfg_attr(feature = "runtime-async-std", async_std::test)]
        #[cfg_attr(feature = "runtime-tokio", tokio::test)]
        async fn $name () -> anyhow::Result<()> {
            let mut conn = connect().await?;

            $(
                let row = sqlx::query(&format!("SELECT {} = ?, ? as _1", $text))
                    .bind($value)
                    .bind($value)
                    .fetch_one(&mut conn)
                    .await?;

                let value = row.get::<$ty, _>("_1");

                assert_eq!(row.get::<i32, _>(0), 1, "value returned from server: {:?}", value);

                assert_eq!($value, value);
            )+

            Ok(())
        }
    }
}

test!(mysql_bool: bool: "false" == false, "true" == true);

test!(mysql_tiny_unsigned: u8: "253" == 253_u8);
test!(mysql_tiny: i8: "5" == 5_i8);

test!(mysql_medium_unsigned: u16: "21415" == 21415_u16);
test!(mysql_short: i16: "21415" == 21415_i16);

test!(mysql_long_unsigned: u32: "2141512" == 2141512_u32);
test!(mysql_long: i32: "2141512" == 2141512_i32);

test!(mysql_longlong_unsigned: u64: "2141512" == 2141512_u64);
test!(mysql_longlong: i64: "2141512" == 2141512_i64);

// `DOUBLE` can be compared with decimal literals just fine but the same can't be said for `FLOAT`
test!(mysql_double: f64: "3.14159265" == 3.14159265f64);

test!(mysql_string: String: "'helloworld'" == "helloworld");

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_bytes() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = &b"Hello, World"[..];

    let rec = sqlx::query!(
        "SELECT (X'48656c6c6f2c20576f726c64' = ?) as _1, CAST(? as BINARY) as _2",
        value,
        value
    )
    .fetch_one(&mut conn)
    .await?;

    assert!(rec._1 != 0);

    let output: Vec<u8> = rec._2;

    assert_eq!(&value[..], &*output);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_float() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = 10.2f32;
    let row = sqlx::query("SELECT ? as _1")
        .bind(value)
        .fetch_one(&mut conn)
        .await?;

    // comparison between FLOAT and literal doesn't work as expected
    // we get implicit widening to DOUBLE which gives a slightly different value
    // however, round-trip does work as expected
    let ret = row.get::<f32, _>("_1");
    assert_eq!(value, ret);

    Ok(())
}
