use sqlx::types::chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::{Connection, PgConnection, Row};

async fn connect() -> anyhow::Result<PgConnection> {
    Ok(PgConnection::open(dotenv::var("DATABASE_URL")?).await?)
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn postgres_chrono_date() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = NaiveDate::from_ymd(2019, 1, 2);

    let row = sqlx::query("SELECT DATE '2019-01-02' = $1, $1")
        .bind(&value)
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_chrono_date_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = NaiveDate::from_ymd(2019, 1, 2).and_hms(5, 10, 20);

    let row = sqlx::query("SELECT '2019-01-02 05:10:20' = $1, $1")
        .bind(&value)
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn postgres_chrono_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = NaiveTime::from_hms_micro(5, 10, 20, 115100);

    let row = sqlx::query("SELECT TIME '05:10:20.115100' = $1, TIME '05:10:20.115100'")
        .bind(&value)
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn postgres_chrono_timestamp_tz() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = DateTime::<Utc>::from_utc(
        NaiveDate::from_ymd(2019, 1, 2).and_hms_micro(5, 10, 20, 115100),
        Utc,
    );

    let row = sqlx::query(
        "SELECT TIMESTAMPTZ '2019-01-02 05:10:20.115100' = $1, TIMESTAMPTZ '2019-01-02 05:10:20.115100'",
    )
    .bind(&value)
    .fetch_one(&mut conn)
    .await?;

    assert!(row.get::<bool, _>(0));

    let out: DateTime<Utc> = row.get(1);
    assert_eq!(value, out);

    Ok(())
}
