use sqlx::{sqlite::Sqlite, Executor};
use sqlx_core::describe::Column;
use sqlx_test::new;

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

    assert_eq!(column_type_names[0], "BIGINT");
    assert_eq!(column_type_names[1], "TEXT");
    assert_eq!(column_type_names[2], "BOOLEAN");
    assert_eq!(column_type_names[3], "BIGINT");

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_expression() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let d = conn.describe("SELECT 1 + 10").await?;
    let columns = d.columns;

    assert_eq!(columns[0].name, "1 + 10");
    assert_eq!(columns[0].not_null, None);

    // SQLite cannot infer types for expressions
    assert_eq!(columns[0].type_info, None);

    Ok(())
}
