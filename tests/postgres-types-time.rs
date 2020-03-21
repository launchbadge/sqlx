use sqlx::types::time::{Date, OffsetDateTime, Time, UtcOffset};
use sqlx::{Connection, PgConnection, Row};

async fn connect() -> anyhow::Result<PgConnection> {
    Ok(PgConnection::open(dotenv::var("DATABASE_URL")?).await?)
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn postgres_timers_date() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = Date::try_from_ymd(2019, 1, 2).unwrap();

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
async fn mysql_timers_date_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = Date::try_from_ymd(2019, 1, 2)
        .unwrap()
        .try_with_hms(5, 10, 20)
        .unwrap();

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
async fn postgres_timers_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = Time::try_from_hms_micro(5, 10, 20, 115100).unwrap();

    let row = sqlx::query!(
        "SELECT TIME '05:10:20.115100' = $1 AS equality, TIME '05:10:20.115100' AS time",
        value
    )
    .fetch_one(&mut conn)
    .await?;

    assert!(row.equality);
    assert_eq!(value, row.time);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn postgres_timers_timestamp_tz() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = Date::try_from_ymd(2019, 1, 2)
        .unwrap()
        .try_with_hms_micro(5, 10, 20, 115100)
        .unwrap()
        .assume_utc();

    let row = sqlx::query(
        "SELECT TIMESTAMPTZ '2019-01-02 05:10:20.115100' = $1, TIMESTAMPTZ '2019-01-02 05:10:20.115100'",
    )
    .bind(&value)
    .fetch_one(&mut conn)
    .await?;

    assert!(row.get::<bool, _>(0));

    let out: OffsetDateTime = row.get(1);
    assert_eq!(value, out);

    let value = Date::try_from_ymd(2019, 1, 2)
        .unwrap()
        .try_with_hms_micro(5, 10, 20, 115100)
        .unwrap()
        .assume_offset(UtcOffset::east_hours(3));

    let row = sqlx::query(
        "SELECT TIMESTAMPTZ '2019-01-02 02:10:20.115100' = $1, TIMESTAMPTZ '2019-01-02 02:10:20.115100'",
    )
    .bind(&value)
    .fetch_one(&mut conn)
    .await?;

    assert!(row.get::<bool, _>(0));

    let out: OffsetDateTime = row.get(1);
    assert_eq!(value, out);

    Ok(())
}
