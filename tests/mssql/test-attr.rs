// The no-arg variant is covered by other tests already.

use sqlx::MssqlPool;

const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("tests/mssql/migrations");

#[sqlx::test]
async fn it_gets_a_pool(pool: MssqlPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    let db_name: String = sqlx::query_scalar("SELECT DB_NAME()")
        .fetch_one(&mut *conn)
        .await?;

    assert!(db_name.starts_with("_sqlx_test_"), "db_name: {:?}", db_name);

    Ok(())
}

// This should apply migrations and then `fixtures/users.sql`
#[sqlx::test(migrations = "tests/mssql/migrations", fixtures("users"))]
async fn it_gets_users(pool: MssqlPool) -> sqlx::Result<()> {
    let usernames: Vec<String> =
        sqlx::query_scalar(r#"SELECT username FROM [user] ORDER BY username"#)
            .fetch_all(&pool)
            .await?;

    assert_eq!(usernames, ["alice", "bob"]);

    let post_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM post")
        .fetch_one(&pool)
        .await?;

    assert_eq!(post_count, 0);

    let comment_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM comment")
        .fetch_one(&pool)
        .await?;

    assert_eq!(comment_count, 0);

    Ok(())
}

#[sqlx::test(migrations = "tests/mssql/migrations", fixtures("users", "posts"))]
async fn it_gets_posts(pool: MssqlPool) -> sqlx::Result<()> {
    let post_contents: Vec<String> =
        sqlx::query_scalar("SELECT content FROM post ORDER BY created_at")
            .fetch_all(&pool)
            .await?;

    assert_eq!(
        post_contents,
        [
            "This new computer is lightning-fast!",
            "@alice is a haxxor :("
        ]
    );

    let comment_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM comment")
        .fetch_one(&pool)
        .await?;

    assert_eq!(comment_count, 0);

    Ok(())
}

// Try `migrator`
#[sqlx::test(migrator = "MIGRATOR", fixtures("users", "posts", "comments"))]
async fn it_gets_comments(pool: MssqlPool) -> sqlx::Result<()> {
    let post_1_comments: Vec<String> =
        sqlx::query_scalar("SELECT content FROM comment WHERE post_id = @p1 ORDER BY created_at")
            .bind(&1)
            .fetch_all(&pool)
            .await?;

    assert_eq!(
        post_1_comments,
        ["lol bet ur still bad, 1v1 me", "you're on!"]
    );

    let post_2_comments: Vec<String> =
        sqlx::query_scalar("SELECT content FROM comment WHERE post_id = @p1 ORDER BY created_at")
            .bind(&2)
            .fetch_all(&pool)
            .await?;

    assert_eq!(post_2_comments, ["lol you're just mad you lost :P"]);

    Ok(())
}
