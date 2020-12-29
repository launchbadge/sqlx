use futures::TryStreamExt;
use sqlx::mysql::{MySql, MySqlConnection, MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Connection, Done, Executor, Row, Statement, TypeInfo};
use sqlx_test::{new, setup_if_needed};
use std::env;

#[sqlx_macros::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    conn.ping().await?;
    conn.close().await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_maths() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let value = sqlx::query("select 1 + CAST(? AS SIGNED)")
        .bind(5_i32)
        .try_map(|row: MySqlRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(6i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fail_at_querying() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let _ = conn.execute(sqlx::query("SELECT 1")).await?;

    // we are testing that this does not cause a panic!
    let _ = conn
        .execute(sqlx::query("SELECT non_existence_table"))
        .await;

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let done = sqlx::query("INSERT INTO users (id) VALUES (?)")
            .bind(index)
            .execute(&mut conn)
            .await?;

        assert_eq!(done.rows_affected(), 1);
    }

    let sum: i32 = sqlx::query("SELECT id FROM users")
        .try_map(|row: MySqlRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes_with_pool() -> anyhow::Result<()> {
    let pool: MySqlPool = MySqlPoolOptions::new()
        .min_connections(2)
        .max_connections(2)
        .test_before_acquire(false)
        .connect(&dotenv::var("DATABASE_URL")?)
        .await?;

    let rows = pool.fetch_all("SELECT 1; SELECT 2").await?;

    assert_eq!(rows.len(), 2);

    let count = pool
        .fetch("SELECT 1; SELECT 2")
        .try_fold(0, |acc, _| async move { Ok(acc + 1) })
        .await?;

    assert_eq!(count, 2);

    Ok(())
}

#[sqlx_macros::test]
async fn it_works_with_cache_disabled() -> anyhow::Result<()> {
    setup_if_needed();

    let mut url = url::Url::parse(&env::var("DATABASE_URL")?)?;
    url.query_pairs_mut()
        .append_pair("statement-cache-capacity", "0");

    let mut conn = MySqlConnection::connect(url.as_ref()).await?;

    for index in 1..=10_i32 {
        let _ = sqlx::query("SELECT ?")
            .bind(index)
            .execute(&mut conn)
            .await?;
    }

    Ok(())
}

#[sqlx_macros::test]
async fn it_drops_results_in_affected_rows() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    // ~1800 rows should be iterated and dropped
    let done = conn
        .execute("select * from mysql.time_zone limit 1575")
        .await?;

    // In MySQL, rows being returned isn't enough to flag it as an _affected_ row
    assert_eq!(0, done.rows_affected());

    Ok(())
}

#[sqlx_macros::test]
async fn it_selects_null() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let (val,): (Option<i32>,) = sqlx::query_as("SELECT NULL").fetch_one(&mut conn).await?;

    assert!(val.is_none());

    let val: Option<i32> = conn.fetch_one("SELECT NULL").await?.try_get(0)?;

    assert!(val.is_none());

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fetch_one_and_ping() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let (_id,): (i32,) = sqlx::query_as("SELECT 1 as id")
        .fetch_one(&mut conn)
        .await?;

    conn.ping().await?;

    let (_id,): (i32,) = sqlx::query_as("SELECT 1 as id")
        .fetch_one(&mut conn)
        .await?;

    Ok(())
}

/// Test that we can interleave reads and writes to the database in one simple query.
#[sqlx_macros::test]
async fn it_interleaves_reads_and_writes() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let mut s = conn.fetch(
        "
CREATE TEMPORARY TABLE messages (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    text TEXT NOT NULL
);

SELECT 'Hello World' as _1;

INSERT INTO messages (text) VALUES ('this is a test');

SELECT id, text FROM messages;
        ",
    );

    let row = s.try_next().await?.unwrap();

    assert!("Hello World" == row.try_get::<&str, _>("_1")?);

    let row = s.try_next().await?.unwrap();

    let id: i64 = row.try_get("id")?;
    let text: &str = row.try_get("text")?;

    assert_eq!(1_i64, id);
    assert_eq!("this is a test", text);

    Ok(())
}

#[sqlx_macros::test]
async fn it_caches_statements() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    for i in 0..2 {
        let row = sqlx::query("SELECT ? AS val")
            .bind(i)
            .persistent(true)
            .fetch_one(&mut conn)
            .await?;

        let val: u32 = row.get("val");

        assert_eq!(i, val);
    }

    assert_eq!(1, conn.cached_statements_size());
    conn.clear_cached_statements().await?;
    assert_eq!(0, conn.cached_statements_size());

    for i in 0..2 {
        let row = sqlx::query("SELECT ? AS val")
            .bind(i)
            .persistent(false)
            .fetch_one(&mut conn)
            .await?;

        let val: u32 = row.get("val");

        assert_eq!(i, val);
    }

    assert_eq!(0, conn.cached_statements_size());

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_bind_null_and_non_null_issue_540() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let row = sqlx::query("SELECT ?, ?")
        .bind(50_i32)
        .bind(None::<i32>)
        .fetch_one(&mut conn)
        .await?;

    let v0: Option<i32> = row.get(0);
    let v1: Option<i32> = row.get(1);

    assert_eq!(v0, Some(50));
    assert_eq!(v1, None);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_bind_only_null_issue_540() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let row = sqlx::query("SELECT ?")
        .bind(None::<i32>)
        .fetch_one(&mut conn)
        .await?;

    let v0: Option<i32> = row.get(0);

    assert_eq!(v0, None);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_bind_and_return_years() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    conn.execute(
        r#"
CREATE TEMPORARY TABLE too_many_years (
    id INT PRIMARY KEY AUTO_INCREMENT,
    the YEAR NOT NULL
);
    "#,
    )
    .await?;

    sqlx::query(
        r#"
INSERT INTO too_many_years ( the ) VALUES ( ? );
    "#,
    )
    .bind(2142)
    .execute(&mut conn)
    .await?;

    let the: u16 = sqlx::query_scalar("SELECT the FROM too_many_years")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(the, 2142);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_prepare_then_execute() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;
    let mut tx = conn.begin().await?;

    let tweet_id: u64 = sqlx::query("INSERT INTO tweet ( text ) VALUES ( 'Hello, World' )")
        .execute(&mut tx)
        .await?
        .last_insert_id();

    let statement = tx.prepare("SELECT * FROM tweet WHERE id = ?").await?;

    assert_eq!(statement.column(0).name(), "id");
    assert_eq!(statement.column(1).name(), "created_at");
    assert_eq!(statement.column(2).name(), "text");
    assert_eq!(statement.column(3).name(), "owner_id");

    assert_eq!(statement.column(0).type_info().name(), "BIGINT");
    assert_eq!(statement.column(1).type_info().name(), "TIMESTAMP");
    assert_eq!(statement.column(2).type_info().name(), "TEXT");
    assert_eq!(statement.column(3).type_info().name(), "BIGINT");

    let row = statement.query().bind(tweet_id).fetch_one(&mut tx).await?;
    let tweet_text: &str = row.try_get("text")?;

    assert_eq!(tweet_text, "Hello, World");

    Ok(())
}

#[tokio::test]
async fn drop_test() -> anyhow::Result<()> {
    const CNT: usize = 222;
    const POOL_SIZE: u32 = 2;

    setup_if_needed();

    let pool = MySqlPoolOptions::new()
        .max_connections(POOL_SIZE)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await?;
    // Create a temporery table and insert a lot of stuff
    {
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS drop_test")
            .execute(&mut conn)
            .await?;
        sqlx::query("CREATE TABLE drop_test (id BIGINT PRIMARY KEY AUTO_INCREMENT)")
            .execute(&mut conn)
            .await?;
        let mut q = "INSERT INTO drop_test () VALUES ()".to_string();
        for _ in 0..CNT {
            q.push_str(", ()");
        }
        sqlx::query(&q).execute(&mut conn).await?;
    }

    // It is somewhat tricky to get the timing right for failures to occur
    // so we repeat the test a number of times
    for _ in 0..50 {
        // Create a bunch of long running jobs
        // that hopefully will be dropped in read
        let s = std::time::Instant::now();
        let mut futures = vec![];
        for i in 0..POOL_SIZE {
            let s = s.clone();
            let pool = pool.clone();
            futures.push(async move {
                let mut conn = pool.acquire().await.unwrap();
                {
                    let mut stream = sqlx::query("SELECT 1 FROM drop_test AS a, drop_test AS b")
                        .fetch(&mut conn);
                    while let Some(_) = stream.try_next().await? {}
                }
                println!("Thread {} finished {}", i, s.elapsed().as_secs_f64());
                Result::<(), anyhow::Error>::Ok(())
            });
        }

        /// Some "feature" in tokio causes the timeout to never occur if the
        /// sleep time is more than one
        #[cfg(feature = "_rt-tokio")]
        fn drop_test_timeout() -> u64 {
            1
        }

        #[cfg(not(feature = "_rt-tokio"))]
        fn drop_test_timeout() -> u64 {
            23
        }

        if let Ok(_) = sqlx_rt::timeout(
            std::time::Duration::from_millis(drop_test_timeout()),
            futures::future::join_all(futures),
        )
        .await
        {
            println!(
                "All queries finished before timeout, this should not happen. We waited {}s",
                s.elapsed().as_secs_f64()
            );
            continue;
        }
        println!("Timeout after {}s", s.elapsed().as_secs_f64());

        // Perform some query and check the result
        let pool = pool.clone();
        let f = async move {
            let mut conn = pool.acquire().await.unwrap();
            let row = sqlx::query("SELECT CAST(SUM(id) AS UNSIGNED) AS s FROM drop_test")
                .fetch_one(&mut conn)
                .await?;
            let s: u64 = row.try_get("s")?;
            assert_eq!(s, ((CNT + 1) * (CNT + 2) / 2) as u64);
            Result::<(), anyhow::Error>::Ok(())
        };
        // We add a timeout here as bugs in the drop handling can cause us to
        // wait for gigabytes of data to be pushed from mysql
        sqlx_rt::timeout(std::time::Duration::from_secs(7), f).await??;
    }

    {
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS drop_test")
            .execute(&mut conn)
            .await?;
    }

    Ok(())
}

// repro is more reliable with the basic scheduler used by `#[tokio::test]`
#[cfg(feature = "_rt-tokio")]
#[tokio::test]
async fn test_issue_622() -> anyhow::Result<()> {
    use std::time::Instant;

    setup_if_needed();

    let pool = MySqlPoolOptions::new()
        .max_connections(1) // also fails with higher counts, e.g. 5
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await?;

    println!("pool state: {:?}", pool);

    let mut handles = vec![];

    // given repro spawned 100 tasks but I found it reliably reproduced with 3
    for i in 0..3 {
        let pool = pool.clone();

        handles.push(sqlx_rt::spawn(async move {
            {
                let mut conn = pool.acquire().await.unwrap();

                let _ = sqlx::query("SELECT 1").fetch_one(&mut conn).await.unwrap();

                // conn gets dropped here and should be returned to the pool
            }

            // (do some other work here without holding on to a connection)
            // this actually fixes the issue, depending on the timeout used
            // sqlx_rt::sleep(Duration::from_millis(500)).await;

            {
                let start = Instant::now();
                match pool.acquire().await {
                    Ok(conn) => {
                        println!("{} acquire took {:?}", i, start.elapsed());
                        drop(conn);
                    }
                    Err(e) => panic!("{} acquire returned error: {} pool state: {:?}", i, e, pool),
                }
            }

            Result::<(), anyhow::Error>::Ok(())
        }));
    }

    futures::future::try_join_all(handles).await?;

    Ok(())
}
