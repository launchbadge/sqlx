use sqlx::{Connection, Postgres, Row};

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        #[async_std::test]
        async fn $name () -> Result<(), String> {
            let mut conn =
                Connection::<Postgres>::open(
                    &dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set")
                ).await.map_err(|e| format!("failed to connect to Postgres: {}", e))?;

            $(
                let row = sqlx::query(&format!("SELECT {} = $1, $1", $text))
                    .bind($value)
                    .fetch_one(&mut conn)
                    .await
                    .map_err(|e| format!("failed to run query: {}", e))?;

                assert!(row.get::<bool>(0));
                assert!($value == row.get::<$ty>(1));
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
