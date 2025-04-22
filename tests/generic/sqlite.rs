mod ansi;

use sqlx::Sqlite;
use sqlx_test::{new, pool, setup_if_needed};

#[sqlx_macros::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let value = ansi::generic_it_connects(&mut conn).await?;

    assert_eq!(2i32, value);

    Ok(())
}
