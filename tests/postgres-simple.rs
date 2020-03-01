use sqlx::{Connect, Executor, Cursor, Row, PgConnection};
use sqlx::postgres::PgRow;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_empty_query() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    let affected = conn.execute("").await?;

    assert_eq!(affected, 0);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_select_1() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    
    let mut cursor = conn.fetch("SELECT 5");
    let row = cursor.next().await?.unwrap();

    assert!(5i32 == row.get::<i32, _>(0)?);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_multi_create_insert() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    
    let mut cursor = conn.fetch("
CREATE TABLE IF NOT EXISTS _sqlx_test_postgres_5112 (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL
);

SELECT 'Hello World';

INSERT INTO _sqlx_test_postgres_5112 (text) VALUES ('this is a test');

SELECT id, text FROM _sqlx_test_postgres_5112;
    ");

    let row = cursor.next().await?.unwrap();

    assert!("Hello World" == row.get::<&str, _>(0)?);

    let row = cursor.next().await?.unwrap();

    let id: i64 = row.get(0)?;
    let text: &str = row.get(1)?;

    assert!(1_i64 == id);
    assert!("this is a test" == text);

    Ok(())
}

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        #[cfg_attr(feature = "runtime-async-std", async_std::test)]
        #[cfg_attr(feature = "runtime-tokio", tokio::test)]
        async fn $name () -> anyhow::Result<()> {
            let mut conn = connect().await?;

            $(
                let rec: $ty = sqlx::query(&format!("SELECT $1 as _1"))
                    .bind($value)
                    .map(|row: PgRow| row.get(0))
                    .fetch_one(&mut conn)
                    .await?;

                assert!($value == rec);
            )+

            Ok(())
        }
    }
}

test!(postgres_simple_bool: bool: "false::boolean" == false, "true::boolean" == true);

test!(postgres_simple_smallint: i16: "821::smallint" == 821_i16);
test!(postgres_simple_int: i32: "94101::int" == 94101_i32);
test!(postgres_simple_bigint: i64: "9358295312::bigint" == 9358295312_i64);

test!(postgres_simple_real: f32: "9419.122::real" == 9419.122_f32);
test!(postgres_simple_double: f64: "939399419.1225182::double precision" == 939399419.1225182_f64);

test!(postgres_simple_text: String: "'this is foo'" == "this is foo", "''" == "");

async fn connect() -> anyhow::Result<PgConnection> {
    let _ = dotenv::dotenv();
    let _ = env_logger::try_init();

    Ok(PgConnection::connect(dotenv::var("DATABASE_URL")?).await?)
}
