use futures::TryStreamExt;
use sqlx::{mysql::MySqlQueryAs, Connection, Executor, MySql, MySqlPool};
use sqlx_test::new;
use std::time::Duration;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_connects() -> anyhow::Result<()> {
    Ok(new::<MySql>().await?.ping().await?)
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_drops_results_in_affected_rows() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    // ~1800 rows should be iterated and dropped
    let affected = conn.execute("select * from mysql.time_zone").await?;

    // In MySQL, rows being returned isn't enough to flag it as an _affected_ row
    assert_eq!(0, affected);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY)
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let cnt = sqlx::query("INSERT INTO users (id) VALUES (?)")
            .bind(index)
            .execute(&mut conn)
            .await?;

        assert_eq!(cnt, 1);
    }

    let sum: i32 = sqlx::query_as("SELECT id FROM users")
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, (x,): (i32,)| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_selects_null() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let (val,): (Option<i32>,) = sqlx::query_as("SELECT NULL").fetch_one(&mut conn).await?;

    assert!(val.is_none());

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_describe() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let _ = conn
        .execute(
            r#"
        CREATE TEMPORARY TABLE describe_test (
            id int primary key auto_increment,
            name text not null,
            hash blob
        )
    "#,
        )
        .await?;

    let describe = conn
        .describe("select nt.*, false from describe_test nt")
        .await?;

    assert_eq!(describe.result_columns[0].non_null, Some(true));
    assert_eq!(describe.result_columns[0].type_info.type_name(), "INT");
    assert_eq!(describe.result_columns[1].non_null, Some(true));
    assert_eq!(describe.result_columns[1].type_info.type_name(), "TEXT");
    assert_eq!(describe.result_columns[2].non_null, Some(false));
    assert_eq!(describe.result_columns[2].type_info.type_name(), "TEXT");
    assert_eq!(describe.result_columns[3].non_null, Some(true));

    let bool_ty_name = describe.result_columns[3].type_info.type_name();

    // MySQL 5.7, 8 and MariaDB 10.1 return BIG_INT, MariaDB 10.4 returns INT (optimization?)
    assert!(
        ["BIG_INT", "INT"].contains(&bool_ty_name),
        "type name returned: {}",
        bool_ty_name
    );

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn pool_immediately_fails_with_db_error() -> anyhow::Result<()> {
    // Malform the database url by changing the password
    let url = dotenv::var("DATABASE_URL")?.replace("password", "not-the-password");

    let pool = MySqlPool::new(&url).await?;

    let res = pool.acquire().await;

    match res {
        Err(sqlx::Error::Database(err)) if err.message().contains("Access denied") => {
            // Access was properly denied
        }

        Err(e) => panic!("unexpected error: {:?}", e),

        Ok(_) => panic!("unexpected ok"),
    }

    Ok(())
}

// run with `cargo test --features mysql -- --ignored --nocapture pool_smoke_test`
#[ignore]
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn pool_smoke_test() -> anyhow::Result<()> {
    use sqlx_core::runtime::{sleep, spawn, timeout};

    eprintln!("starting pool");

    let pool = MySqlPool::builder()
        .connect_timeout(Duration::from_secs(5))
        .min_size(5)
        .max_size(10)
        .build(&dotenv::var("DATABASE_URL")?)
        .await?;

    // spin up more tasks than connections available, and ensure we don't deadlock
    for i in 0..20 {
        let pool = pool.clone();
        spawn(async move {
            loop {
                if let Err(e) = sqlx::query("select 1 + 1").execute(&mut &pool).await {
                    eprintln!("pool task {} dying due to {}", i, e);
                    break;
                }
            }
        });
    }

    for _ in 0..5 {
        let pool = pool.clone();
        spawn(async move {
            while !pool.is_closed() {
                // drop acquire() futures in a hot loop
                // https://github.com/launchbadge/sqlx/issues/83
                drop(pool.acquire());
            }
        });
    }

    eprintln!("sleeping for 30 seconds");

    sleep(Duration::from_secs(30)).await;

    assert_eq!(pool.size(), 10);

    eprintln!("closing pool");

    timeout(Duration::from_secs(30), pool.close()).await?;

    eprintln!("pool closed successfully");

    Ok(())
}
