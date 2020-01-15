use sqlx::types::chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::{mysql::MySqlConnection, Connection, Row};

async fn connect() -> anyhow::Result<MySqlConnection> {
    Ok(MySqlConnection::open(dotenv::var("DATABASE_URL")?).await?)
}

#[async_std::test]
async fn mysql_chrono_date() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = NaiveDate::from_ymd(2019, 1, 2);

    let row = sqlx::query!(
        "SELECT (DATE '2019-01-02' = ?) as _1, CAST(? AS DATE) as _2",
        value,
        value
    )
    .fetch_one(&mut conn)
    .await?;

    assert!(row._1 != 0);
    assert_eq!(value, row._2);

    Ok(())
}

#[async_std::test]
async fn mysql_chrono_date_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = NaiveDate::from_ymd(2019, 1, 2).and_hms(5, 10, 20);

    let row = sqlx::query("SELECT '2019-01-02 05:10:20' = ?, ?")
        .bind(&value)
        .bind(&value)
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}

#[async_std::test]
async fn mysql_chrono_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = NaiveTime::from_hms_micro(5, 10, 20, 115100);

    let row = sqlx::query("SELECT TIME '05:10:20.115100' = ?, TIME '05:10:20.115100'")
        .bind(&value)
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}

#[async_std::test]
async fn mysql_chrono_timestamp() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = DateTime::<Utc>::from_utc(
        NaiveDate::from_ymd(2019, 1, 2).and_hms_micro(5, 10, 20, 115100),
        Utc,
    );

    let row = sqlx::query(
        "SELECT TIMESTAMP '2019-01-02 05:10:20.115100' = ?, TIMESTAMP '2019-01-02 05:10:20.115100'",
    )
    .bind(&value)
    .fetch_one(&mut conn)
    .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}
