use sqlx::describe::Column;
use sqlx::error::DatabaseError;
use sqlx::sqlite::{SqliteConnectOptions, SqliteError};
use sqlx::{sqlite::Sqlite, Executor};
use sqlx::{Connect, SqliteConnection, TypeInfo};
use sqlx_test::new;
use std::env;

fn type_names(columns: &[Column<Sqlite>]) -> Vec<String> {
    columns
        .iter()
        .filter_map(|col| Some(col.type_info.as_ref()?.to_string()))
        .collect()
}

#[sqlx_macros::test]
async fn it_describes_simple() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let d = conn.describe("SELECT * FROM tweet").await?;

    let columns = d.columns;

    assert_eq!(columns[0].name, "id");
    assert_eq!(columns[1].name, "text");
    assert_eq!(columns[2].name, "is_sent");
    assert_eq!(columns[3].name, "owner_id");

    assert_eq!(columns[0].not_null, Some(true));
    assert_eq!(columns[1].not_null, Some(true));
    assert_eq!(columns[2].not_null, Some(true));
    assert_eq!(columns[3].not_null, Some(false)); // owner_id

    let column_type_names = type_names(&columns);

    assert_eq!(column_type_names[0], "INTEGER");
    assert_eq!(column_type_names[1], "TEXT");
    assert_eq!(column_type_names[2], "BOOLEAN");
    assert_eq!(column_type_names[3], "INTEGER");

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_expression() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let d = conn
        .describe("SELECT 1 + 10, 5.12 * 2, 'Hello', x'deadbeef'")
        .await?;

    let columns = d.columns;

    assert_eq!(columns[0].type_info.as_ref().unwrap().name(), "INTEGER");
    assert_eq!(columns[0].name, "1 + 10");
    assert_eq!(columns[0].not_null, None);

    assert_eq!(columns[1].type_info.as_ref().unwrap().name(), "REAL");
    assert_eq!(columns[1].name, "5.12 * 2");
    assert_eq!(columns[1].not_null, None);

    assert_eq!(columns[2].type_info.as_ref().unwrap().name(), "TEXT");
    assert_eq!(columns[2].name, "'Hello'");
    assert_eq!(columns[2].not_null, None);

    assert_eq!(columns[3].type_info.as_ref().unwrap().name(), "BLOB");
    assert_eq!(columns[3].name, "x'deadbeef'");
    assert_eq!(columns[3].not_null, None);

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_expression_from_empty_table() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    conn.execute("CREATE TEMP TABLE _temp_empty ( name TEXT, a INT )")
        .await?;

    let d = conn
        .describe("SELECT COUNT(*), a + 1, name, 5.12, 'Hello' FROM _temp_empty")
        .await?;

    assert_eq!(d.columns[0].type_info.as_ref().unwrap().name(), "INTEGER");
    assert_eq!(d.columns[1].type_info.as_ref().unwrap().name(), "INTEGER");
    assert_eq!(d.columns[2].type_info.as_ref().unwrap().name(), "TEXT");
    assert_eq!(d.columns[3].type_info.as_ref().unwrap().name(), "REAL");
    assert_eq!(d.columns[4].type_info.as_ref().unwrap().name(), "TEXT");

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_insert() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let d = conn
        .describe("INSERT INTO tweet (id, text) VALUES (2, 'Hello')")
        .await?;

    assert_eq!(d.columns.len(), 0);

    let d = conn
        .describe("INSERT INTO tweet (id, text) VALUES (2, 'Hello'); SELECT last_insert_rowid();")
        .await?;

    assert_eq!(d.columns.len(), 1);
    assert_eq!(d.columns[0].type_info.as_ref().unwrap().name(), "INTEGER");

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_insert_with_read_only() -> anyhow::Result<()> {
    sqlx_test::setup_if_needed();

    let mut options: SqliteConnectOptions = env::var("DATABASE_URL")?.parse().unwrap();
    options = options.read_only(true);

    let mut conn = SqliteConnection::connect_with(&options).await?;

    let d = conn
        .describe("INSERT INTO tweet (id, text) VALUES (2, 'Hello')")
        .await?;

    assert_eq!(d.columns.len(), 0);

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_bad_statement() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let err = conn.describe("SELECT 1 FROM not_found").await.unwrap_err();
    let err = err
        .as_database_error()
        .unwrap()
        .downcast_ref::<SqliteError>();

    assert_eq!(err.message(), "no such table: not_found");
    assert_eq!(err.code().as_deref(), Some("1"));

    Ok(())
}
