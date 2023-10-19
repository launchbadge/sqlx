use sqlx::{Any, Sqlite};
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_encodes_bool_with_any() -> anyhow::Result<()> {
    sqlx::any::install_default_drivers();
    let mut conn = new::<Any>().await?;

    let res = sqlx::query("INSERT INTO accounts (name, is_active) VALUES (?, ?)")
        .bind("Harrison Ford")
        .bind(true)
        .execute(&mut conn)
        .await
        .expect("failed to encode bool");
    assert_eq!(res.rows_affected(), 1);

    Ok(())
}
