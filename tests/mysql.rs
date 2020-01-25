use futures::TryStreamExt;
use sqlx::{Connection as _, Executor as _, MySqlConnection, MySqlPool, Row as _};
use std::time::Duration;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let row = sqlx::query("select 1 + 1").fetch_one(&mut conn).await?;

    assert_eq!(2, row.get(0));

    conn.close().await?;

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let _ = conn
        .send(
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

    let sum: i32 = sqlx::query("SELECT id FROM users")
        .fetch(&mut conn)
        .try_fold(
            0_i32,
            |acc, x| async move { Ok(acc + x.get::<i32, _>("id")) },
        )
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_selects_null() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let row = sqlx::query("SELECT NULL").fetch_one(&mut conn).await?;

    let val: Option<i32> = row.get(0);

    assert!(val.is_none());

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_describe() -> anyhow::Result<()> {
    use sqlx::describe::Nullability::*;

    let mut conn = connect().await?;

    let _ = conn
        .send(
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

    assert_eq!(describe.result_columns[0].nullability, NonNull);
    assert_eq!(describe.result_columns[0].type_info.type_name(), "INT");
    assert_eq!(describe.result_columns[1].nullability, NonNull);
    assert_eq!(describe.result_columns[1].type_info.type_name(), "TEXT");
    assert_eq!(describe.result_columns[2].nullability, Nullable);
    assert_eq!(describe.result_columns[2].type_info.type_name(), "TEXT");
    assert_eq!(describe.result_columns[3].nullability, NonNull);

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
    let url = url()?.replace("password", "not-the-password");

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
                if let Err(e) = sqlx::query("select 1 + 1").fetch_one(&mut &pool).await {
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

fn url() -> anyhow::Result<String> {
    Ok(dotenv::var("DATABASE_URL")?)
}

async fn connect() -> anyhow::Result<MySqlConnection> {
    Ok(MySqlConnection::open(url()?).await?)
}
