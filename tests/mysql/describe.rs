use sqlx::mysql::MySql;
use sqlx::{Column, Executor, TypeInfo};
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_describes_simple() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let d = conn.describe_full("SELECT * FROM tweet").await?;

    assert_eq!(d.column(0).name(), "id");
    assert_eq!(d.column(1).name(), "created_at");
    assert_eq!(d.column(2).name(), "text");
    assert_eq!(d.column(3).name(), "owner_id");

    assert_eq!(d.nullable(0), Some(false));
    assert_eq!(d.nullable(1), Some(false));
    assert_eq!(d.nullable(2), Some(false));
    assert_eq!(d.nullable(3), Some(true));

    assert_eq!(d.column(0).type_info().name(), "BIGINT");
    assert_eq!(d.column(1).type_info().name(), "TIMESTAMP");
    assert_eq!(d.column(2).type_info().name(), "TEXT");
    assert_eq!(d.column(3).type_info().name(), "BIGINT");

    Ok(())
}

#[sqlx_macros::test]
async fn uses_alias_name() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let d = conn
        .describe_full("SELECT text AS tweet_text FROM tweet")
        .await?;

    assert_eq!(d.column(0).name(), "tweet_text");

    Ok(())
}
