use anyhow::Context;
use jiff_sqlx::ToSqlx;
use sqlx::{Connection, PgConnection};

#[derive(Debug)]
struct DateTimeTypes {
    date_column: jiff::civil::Date,
    time_column: jiff::civil::Time,
    datetime_column: jiff::civil::DateTime,
    timestamp_column: jiff::Timestamp,
    span_column: jiff::Span,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut conn = PgConnection::connect(
        &dotenvy::var("DATABASE_URL").context("DATABASE_URL must be set")?
    ).await
        .context("error connecting to DATABASE_URL")?;

    sqlx::migrate!().run(&mut conn).await?;

    // No overrides necessary, despite `jiff` not being directly integrated with SQLx.
    //
    // NOTE: `jiff_sqlx::Span` does not implement `Encode`:
    // https://docs.rs/jiff-sqlx/latest/jiff_sqlx/struct.Span.html#postgresql-limited-support
    //
    // Intervals in Postgres have slightly different semantics than `jiff::Span` and so are not
    // directly interchangeable. For demonstration purposes, we'll just insert a constant string.
    sqlx::query!(
        "INSERT INTO date_time_types ( \
            date_column, time_column, datetime_column, timestamp_column, span_column \
         ) VALUES ($1, $2, $3, $4, $5::text::interval)",
        jiff::civil::date(2025, 04, 11).to_sqlx(),
        jiff::civil::time(15, 39, 54, 0).to_sqlx(),
        jiff::civil::datetime(2025, 04, 11, 15, 40, 22, 0).to_sqlx(),
        jiff::Timestamp::now().to_sqlx(),
        "1 hour"
    )
        .execute(&mut conn)
        .await?;

    let row = sqlx::query_as!(
        DateTimeTypes,
        // `SELECT *` may be applicable here, but is not recommended because the order of columns
        // could change from compile time to runtime and cause errors.
        "SELECT date_column, time_column, datetime_column, timestamp_column, span_column FROM date_time_types"
    )
        .fetch_one(&mut conn)
        .await?;

    println!("Row from database: {row:#?}");

    conn.close().await?;

    Ok(())
}
