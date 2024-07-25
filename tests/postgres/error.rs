use sqlx::{error::ErrorKind, postgres::Postgres, Connection};
use sqlx_core::executor::Executor;
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_fails_with_unique_violation() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    let mut tx = conn.begin().await?;

    sqlx::query("INSERT INTO tweet(id, text, owner_id) VALUES (1, 'Foo', 1);")
        .execute(&mut *tx)
        .await?;

    let res: Result<_, sqlx::Error> = sqlx::query("INSERT INTO tweet VALUES (1, NOW(), 'Foo', 1);")
        .execute(&mut *tx)
        .await;
    let err = res.unwrap_err();

    let err = err.into_database_error().unwrap();

    assert_eq!(err.kind(), ErrorKind::UniqueViolation);

    Ok(())
}

#[sqlx_macros::test]
async fn it_fails_with_foreign_key_violation() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    let mut tx = conn.begin().await?;

    let res: Result<_, sqlx::Error> =
        sqlx::query("INSERT INTO tweet_reply (tweet_id, text) VALUES (1, 'Reply!');")
            .execute(&mut *tx)
            .await;
    let err = res.unwrap_err();

    let err = err.into_database_error().unwrap();

    assert_eq!(err.kind(), ErrorKind::ForeignKeyViolation);

    Ok(())
}

#[sqlx_macros::test]
async fn it_fails_with_not_null_violation() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    let mut tx = conn.begin().await?;

    let res: Result<_, sqlx::Error> = sqlx::query("INSERT INTO tweet (text) VALUES (null);")
        .execute(&mut *tx)
        .await;
    let err = res.unwrap_err();

    let err = err.into_database_error().unwrap();

    assert_eq!(err.kind(), ErrorKind::NotNullViolation);

    Ok(())
}

#[sqlx_macros::test]
async fn it_fails_with_check_violation() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    let mut tx = conn.begin().await?;

    let res: Result<_, sqlx::Error> =
        sqlx::query("INSERT INTO products VALUES (1, 'Product 1', 0);")
            .execute(&mut *tx)
            .await;
    let err = res.unwrap_err();

    let err = err.into_database_error().unwrap();

    assert_eq!(err.kind(), ErrorKind::CheckViolation);

    Ok(())
}


#[sqlx::test]
async fn test_error_includes_position() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let err: sqlx::Error = conn
        .prepare("SELECT bar.foo as foo\nFORM bar")
        .await
        .unwrap_err();

    let sqlx::Error::Database(dbe) = err else {
        panic!("unexpected error kind {err:?}")
    };

    let pos = dbe.position().unwrap();

    assert_eq!(pos.line, 2);
    assert_eq!(pos.column, 1);
    assert!(
        dbe.to_string().contains("line 2, column 1"),
        "{:?}",
        dbe.to_string()
    );

    let err: sqlx::Error = sqlx::query("SELECT bar.foo as foo\r\nFORM bar")
        .execute(&mut conn)
        .await
        .unwrap_err();

    let sqlx::Error::Database(dbe) = err else {
        panic!("unexpected error kind {err:?}")
    };

    let pos = dbe.position().unwrap();

    assert_eq!(pos.line, 2);
    assert_eq!(pos.column, 1);
    assert!(
        dbe.to_string().contains("line 2, column 1"),
        "{:?}",
        dbe.to_string()
    );

    let err: sqlx::Error = sqlx::query(
        "SELECT foo\r\nFROM bar\r\nINNER JOIN baz USING (foo)\r\nWHERE foo=1 ADN baz.foo = 2",
    )
        .execute(&mut conn)
        .await
        .unwrap_err();

    let sqlx::Error::Database(dbe) = err else {
        panic!("unexpected error kind {err:?}")
    };

    let pos = dbe.position().unwrap();

    assert_eq!(pos.line, 4);
    assert_eq!(pos.column, 13);
    assert!(
        dbe.to_string().contains("line 4, column 13"),
        "{:?}",
        dbe.to_string()
    );

    Ok(())
}