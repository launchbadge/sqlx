use sqlx::{Connection, MariaDb, Row};
use std::env;

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        #[async_std::test]
        async fn $name () -> sqlx::Result<()> {
            let mut conn =
                Connection::<MariaDb>::open(&env::var("DATABASE_URL").unwrap()).await?;

            $(
                let row = sqlx::query(&format!("SELECT {} = ?, ?", $text))
                    .bind($value)
                    .bind($value)
                    .fetch_one(&mut conn)
                    .await?;

                assert_eq!(row.get::<i32>(0), 1);
                assert!($value == row.get::<$ty>(1));
            )+

            Ok(())
        }
    }
}

test!(mysql_bool: bool: "false" == false, "true" == true);
test!(mysql_long: i32: "2141512" == 2141512_i32);


