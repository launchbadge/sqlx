use sqlx::mysql::MySql;
use sqlx::Executor;
use sqlx_core::describe::Column;
use sqlx_test::new;

fn type_names(columns: &[Column<MySql>]) -> Vec<String> {
    columns
        .iter()
        .filter_map(|col| Some(col.type_info.as_ref()?.to_string()))
        .collect()
}

#[sqlx_macros::test]
async fn it_describes_simple() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let d = conn.describe("SELECT * FROM tweet").await?;
    let columns = d.columns;

    assert_eq!(columns[0].name, "id");
    assert_eq!(columns[1].name, "created_at");
    assert_eq!(columns[2].name, "text");
    assert_eq!(columns[3].name, "owner_id");

    assert_eq!(columns[0].not_null, Some(true));
    assert_eq!(columns[1].not_null, Some(true));
    assert_eq!(columns[2].not_null, Some(true));
    assert_eq!(columns[3].not_null, Some(false));

    let column_type_names = type_names(&columns);

    assert_eq!(column_type_names[0], "BIGINT");
    assert_eq!(column_type_names[1], "TIMESTAMP");
    assert_eq!(column_type_names[2], "TEXT");
    assert_eq!(column_type_names[3], "BIGINT");

    Ok(())
}
