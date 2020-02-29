use sqlx::types::time::{Date, OffsetDateTime, Time, UtcOffset};
use sqlx::{mysql::MySqlConnection, Connection, Row};

async fn connect() -> anyhow::Result<MySqlConnection> {
    Ok(MySqlConnection::open(dotenv::var("DATABASE_URL")?).await?)
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_timers_date() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    // TODO: maybe use macro here? but is it OK to include `time` as test dependency?
    let value = Date::try_from_ymd(2019, 1, 2).unwrap();

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

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_timers_date_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = Date::try_from_ymd(2019, 1, 2)
        .unwrap()
        .try_with_hms(5, 10, 20)
        .unwrap();

    let row = sqlx::query("SELECT '2019-01-02 05:10:20' = ?, ?")
        .bind(&value)
        .bind(&value)
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_timers_time() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = Time::try_from_hms_micro(5, 10, 20, 115100).unwrap();

    let row = sqlx::query("SELECT TIME '05:10:20.115100' = ?, TIME '05:10:20.115100'")
        .bind(&value)
        .fetch_one(&mut conn)
        .await?;

    assert!(row.get::<bool, _>(0));
    assert_eq!(value, row.get(1));

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn mysql_timers_timestamp() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = Date::try_from_ymd(2019, 1, 2)
        .unwrap()
        .try_with_hms_micro(5, 10, 20, 115100)
        .unwrap()
        .assume_utc();

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
