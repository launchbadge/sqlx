use sqlx::{postgres::Postgres, Executor};
use sqlx_core::describe::Column;
use sqlx_test::new;

fn type_names(columns: &[Column<Postgres>]) -> Vec<String> {
    columns
        .iter()
        .filter_map(|col| Some(col.type_info.as_ref()?.to_string()))
        .collect()
}

#[sqlx_macros::test]
async fn it_describes_simple() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

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

    assert_eq!(column_type_names[0], "INT8");
    assert_eq!(column_type_names[1], "TIMESTAMPTZ");
    assert_eq!(column_type_names[2], "TEXT");
    assert_eq!(column_type_names[3], "INT8");

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_expression() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let d = conn.describe("SELECT 1::int8 + 10").await?;
    let columns = d.columns;

    // ?column? will cause the macro to emit an error ad ask the user to explicitly name the type
    assert_eq!(columns[0].name, "?column?");

    // postgres cannot infer nullability from an expression
    // this will cause the macro to emit `Option<_>`
    assert_eq!(columns[0].not_null, None);

    let column_type_names = type_names(&columns);

    assert_eq!(column_type_names[0], "INT8");

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_enum() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let d = conn.describe("SELECT 'open'::status as _1").await?;
    let columns = d.columns;

    assert_eq!(columns[0].name, "_1");
    assert_eq!(columns[0].not_null, None);

    let ty = columns[0].type_info.as_ref().unwrap();

    assert_eq!(ty.to_string(), "status");
    assert_eq!(
        format!("{:?}", ty.__kind()),
        r#"Enum(["new", "open", "closed"])"#
    );

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_record() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let d = conn.describe("SELECT (true, 10::int2)").await?;
    let columns = d.columns;

    let ty = columns[0].type_info.as_ref().unwrap();
    assert_eq!(ty.to_string(), "RECORD");

    Ok(())
}

#[sqlx_macros::test]
async fn it_describes_composite() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let d = conn
        .describe("SELECT ROW('name',10,500)::inventory_item")
        .await?;

    let columns = d.columns;

    let ty = columns[0].type_info.as_ref().unwrap();

    assert_eq!(ty.to_string(), "inventory_item");

    assert_eq!(
        format!("{:?}", ty.__kind()),
        r#"Composite([("name", PgTypeInfo(Text)), ("supplier_id", PgTypeInfo(Int4)), ("price", PgTypeInfo(Int8))])"#
    );

    Ok(())
}
