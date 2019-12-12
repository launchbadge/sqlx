use sqlx::{Connection, MariaDb, Row};

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        #[async_std::test]
        async fn $name () -> sqlx::Result<()> {
            let mut conn =
                Connection::<MariaDb>::open(
                    &dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set")
                ).await?;

            $(
                let row = sqlx::query(&format!("SELECT {} = ?, ?", $text))
                    .bind($value)
                    .bind($value)
                    .fetch_one(&mut conn)
                    .await?;

                assert_eq!(row.get::<i32>(0), 1);
                let value = row.get::<$ty>(1);
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
