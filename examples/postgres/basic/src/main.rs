use sqlx::{Connect, Connection, Cursor, Executor, PgConnection, Row};
use std::convert::TryInto;
use std::time::Instant;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let mut conn = PgConnection::connect("postgres://").await?;

    let mut rows = sqlx::query("SELECT definition FROM pg_database")
        .execute(&mut conn)
        .await?;

    // let start = Instant::now();
    // while let Some(row) = cursor.next().await? {
    //     // let raw = row.try_get(0)?.unwrap();
    //
    //     // println!("hai: {:?}", raw);
    // }

    println!("?? = {}", rows);

    // conn.close().await?;

    Ok(())
}
