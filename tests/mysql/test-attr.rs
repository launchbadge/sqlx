// The no-arg variant is covered by other tests already.

use sqlx_oldapi::MySqlPool;

const MIGRATOR: sqlx_oldapi::migrate::Migrator = sqlx_oldapi::migrate!("tests/mysql/migrations");

#[sqlx_oldapi::test]
async fn it_gets_a_pool(pool: MySqlPool) -> sqlx_oldapi::Result<()> {
    let mut conn = pool.acquire().await?;

    let db_name: String = sqlx_oldapi::query_scalar("select database()")
        .fetch_one(&mut conn)
        .await?;

    assert!(
        db_name.starts_with("_sqlx_test_database_"),
        "db_name: {:?}",
        db_name
    );

    Ok(())
}

// This should apply migrations and then `fixtures/users.sql`
#[sqlx_oldapi::test(migrations = "tests/mysql/migrations", fixtures("users"))]
async fn it_gets_users(pool: MySqlPool) -> sqlx_oldapi::Result<()> {
    let usernames: Vec<String> =
        sqlx_oldapi::query_scalar(r#"SELECT username FROM user ORDER BY username"#)
            .fetch_all(&pool)
            .await?;

    assert_eq!(usernames, ["alice", "bob"]);

    let post_exists: bool = sqlx_oldapi::query_scalar("SELECT exists(SELECT 1 FROM post)")
        .fetch_one(&pool)
        .await?;

    assert!(!post_exists);

    let comment_exists: bool = sqlx_oldapi::query_scalar("SELECT exists(SELECT 1 FROM comment)")
        .fetch_one(&pool)
        .await?;

    assert!(!comment_exists);

    Ok(())
}

#[sqlx_oldapi::test(migrations = "tests/mysql/migrations", fixtures("users", "posts"))]
async fn it_gets_posts(pool: MySqlPool) -> sqlx_oldapi::Result<()> {
    let post_contents: Vec<String> =
        sqlx_oldapi::query_scalar("SELECT content FROM post ORDER BY created_at")
            .fetch_all(&pool)
            .await?;

    assert_eq!(
        post_contents,
        [
            "This new computer is lightning-fast!",
            "@alice is a haxxor :("
        ]
    );

    let comment_exists: bool = sqlx_oldapi::query_scalar("SELECT exists(SELECT 1 FROM comment)")
        .fetch_one(&pool)
        .await?;

    assert!(!comment_exists);

    Ok(())
}

// Try `migrator`
#[sqlx_oldapi::test(migrator = "MIGRATOR", fixtures("users", "posts", "comments"))]
async fn it_gets_comments(pool: MySqlPool) -> sqlx_oldapi::Result<()> {
    let post_1_comments: Vec<String> = sqlx_oldapi::query_scalar(
        "SELECT content FROM comment WHERE post_id = ? ORDER BY created_at",
    )
    .bind(&1)
    .fetch_all(&pool)
    .await?;

    assert_eq!(
        post_1_comments,
        ["lol bet ur still bad, 1v1 me", "you're on!"]
    );

    let post_2_comments: Vec<String> = sqlx_oldapi::query_scalar(
        "SELECT content FROM comment WHERE post_id = ? ORDER BY created_at",
    )
    .bind(&2)
    .fetch_all(&pool)
    .await?;

    assert_eq!(post_2_comments, ["lol you're just mad you lost :P"]);

    Ok(())
}
