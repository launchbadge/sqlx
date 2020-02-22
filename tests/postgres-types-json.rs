use sqlx::{postgres::{PgConnection, Json, Jsonb}, Connection as _, Row};
use serde::{Deserialize, Serialize};

async fn connect() -> anyhow::Result<PgConnection> {
    Ok(PgConnection::open(dotenv::var("DATABASE_URL")?).await?)
}

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        mod $name {
            use super::*;

            #[cfg_attr(feature = "runtime-async-std", async_std::test)]
            #[cfg_attr(feature = "runtime-tokio", tokio::test)]
            async fn json () -> anyhow::Result<()> {
                let mut conn = connect().await?;

                // Always use jsonb for the comparison, as json does not support equality
                $(
                    let row = sqlx::query(&format!("SELECT {}::json::jsonb = $1::jsonb, $1 as _1", $text))
                        .bind(Json($value))
                        .fetch_one(&mut conn)
                        .await?;

                    assert!(row.get::<bool, _>(0));
                    assert!(Json($value) == row.get::<Json<$ty>, _>("_1"));
                )+

                Ok(())
            }
            #[cfg_attr(feature = "runtime-async-std", async_std::test)]
            #[cfg_attr(feature = "runtime-tokio", tokio::test)]
            async fn jsonb () -> anyhow::Result<()> {
                let mut conn = connect().await?;

                $(
                    let row = sqlx::query(&format!("SELECT {}::jsonb = $1::jsonb, $1 as _1", $text))
                        .bind(Jsonb($value))
                        .fetch_one(&mut conn)
                        .await?;

                    assert!(row.get::<bool, _>(0));
                    assert!(Jsonb($value) == row.get::<Jsonb<$ty>, _>("_1"));
                )+

                Ok(())
            }
        }
    }
}

test!(postgres_json_string: String: "'\"Hello, World!\"'" == "Hello, World!".to_string());

test!(postgres_json_emoji_simple: String: "'\"😎\"'" == "😎".to_string());
test!(postgres_json_emoji_multi: String: "'\"🙋‍♀️\"'" == "🙋‍♀️".to_string());

test!(postgres_json_vec: Vec<String>: "'[\"Hello\", \"World!\"]'" == vec!["Hello".to_string(), "World!".to_string()]);

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Friend {
    name: String,
    age: u32,
}

test!(postgres_json_struct: Friend: "'{\"name\":\"Joe\",\"age\":33}'" == Friend { name: "Joe".to_string(), age: 33 });

