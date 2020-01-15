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

                assert_eq!(row.get::<i32, _>(0), 1);

                let value = row.get::<$ty, _>("_1");

                assert!($value == value);
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

test!(mysql_string: String: "'helloworld'" == "helloworld");

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_bytes() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = b"Hello, World";

    let row = sqlx::query("SELECT X'48656c6c6f2c20576f726c64' = ?, ?")
        .bind(&value[..])
        .bind(&value[..])
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));

    let output: Vec<u8> = row.get(1);

    assert_eq!(&value[..], &*output);

    Ok(())
}
