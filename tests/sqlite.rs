use futures::TryStreamExt;
use sqlx::{sqlite::SqliteQueryAs, Connect, Connection, Executor, Sqlite, SqliteConnection};
use sqlx_test::new;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_connects() -> anyhow::Result<()> {
    Ok(new::<Sqlite>().await?.ping().await?)
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_fails_to_connect() -> anyhow::Result<()> {
    // empty connection string
    assert!(SqliteConnection::connect("").await.is_err());
    assert!(
        SqliteConnection::connect("sqlite:///please_do_not_run_sqlx_tests_as_root")
            .await
            .is_err()
    );

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_fails_to_parse() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;
    let res = conn.execute("SEELCT 1").await;

    assert!(res.is_err());

    let err = res.unwrap_err().to_string();

    assert_eq!("near \"SEELCT\": syntax error", err);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

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
async fn it_can_execute_multiple_statements() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let affected = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY, other INTEGER);
INSERT INTO users DEFAULT VALUES;
            "#,
        )
        .await?;

    assert_eq!(affected, 1);

    for index in 2..5_i32 {
        let (id, other): (i32, i32) = sqlx::query_as(
            r#"
INSERT INTO users (other) VALUES (?);
SELECT id, other FROM users WHERE id = last_insert_rowid();
            "#,
        )
        .bind(index)
        .fetch_one(&mut conn)
        .await?;

        assert_eq!(id, index);
        assert_eq!(other, index);
    }

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_describes() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE describe_test (
    _1 int primary key,
    _2 text not null,
    _3 blob,
    _4 boolean,
    _5 float,
    _6 varchar(255),
    _7 double,
    _8 bigint
)
            "#,
        )
        .await?;

    let describe = conn
        .describe("select nt.*, false from describe_test nt")
        .await?;

    assert_eq!(
        describe.result_columns[0]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "INTEGER"
    );
    assert_eq!(
        describe.result_columns[1]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "TEXT"
    );
    assert_eq!(
        describe.result_columns[2]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "BLOB"
    );
    assert_eq!(
        describe.result_columns[3]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "BOOLEAN"
    );
    assert_eq!(
        describe.result_columns[4]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "DOUBLE"
    );
    assert_eq!(
        describe.result_columns[5]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "TEXT"
    );
    assert_eq!(
        describe.result_columns[6]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "DOUBLE"
    );
    assert_eq!(
        describe.result_columns[7]
            .type_info
            .as_ref()
            .unwrap()
            .to_string(),
        "INTEGER"
    );

    // Expressions can not be described
    assert!(describe.result_columns[8].type_info.is_none());

    Ok(())
}
