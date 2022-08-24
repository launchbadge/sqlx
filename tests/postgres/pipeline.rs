// Test PgExtendedQueryPipeline

use sqlx::postgres::PgExtendedQueryPipeline;
use sqlx::PgPool;
use uuid::{uuid, Uuid};

const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("tests/postgres/migrations");

async fn cleanup_test_data(
    pool: &PgPool,
    user_id: Uuid,
    post_id: Uuid,
    comment_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM comment WHERE comment_id = $1")
        .bind(comment_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM post WHERE post_id = $1")
        .bind(post_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM \"user\" WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// Ensure the test data exists or not
//
// not_exists == true => the test data shouldn't exist
// not_exists == false => the test data is expected to exist
async fn ensure_test_data(
    not_exists: bool,
    user_id: Uuid,
    post_id: Uuid,
    comment_id: Uuid,
    pool: &PgPool,
) -> sqlx::Result<()> {
    let user_exists_query =
        sqlx::query_scalar("SELECT exists(SELECT 1 FROM \"user\" WHERE user_id = $1)")
            .bind(user_id);
    let post_exists_query =
        sqlx::query_scalar("SELECT exists(SELECT 1 FROM post WHERE post_id = $1)").bind(post_id);
    let comment_exists_query =
        sqlx::query_scalar("SELECT exists(SELECT 1 FROM comment WHERE comment_id = $1)")
            .bind(comment_id);

    let user_exists: bool = user_exists_query.fetch_one(pool).await?;
    assert!(not_exists ^ user_exists);

    let post_exists: bool = post_exists_query.fetch_one(pool).await?;
    assert!(not_exists ^ post_exists);

    let comment_exists: bool = comment_exists_query.fetch_one(pool).await?;
    assert!(not_exists ^ comment_exists);
    Ok(())
}

#[sqlx::test(migrations = "tests/postgres/migrations")]
async fn it_executes_pipeline(pool: PgPool) -> sqlx::Result<()> {
    // 0. ensure the clean state

    let user_id = uuid!("6592b7c0-b531-4613-ace5-94246b7ce0c3");
    let post_id = uuid!("252c1d98-a9b0-4f18-8298-e59058bdfe16");
    let comment_id = uuid!("fbbbb7dc-dc6f-4649-b663-8d3636035164");

    cleanup_test_data(&pool, user_id, post_id, comment_id).await?;
    ensure_test_data(true, user_id, post_id, comment_id, &pool).await?;

    // 1. construct pipeline of 3 inserts

    const EXPECTED_QUERIES_IN_PIPELINE: usize = 3;

    // query with parameters
    let user_insert_query = sqlx::query(
        "
        INSERT INTO \"user\" (user_id, username)
        VALUES
        ($1, $2)
    ",
    )
    .bind(user_id)
    .bind("alice");

    let mut pipeline =
        PgExtendedQueryPipeline::<EXPECTED_QUERIES_IN_PIPELINE>::from(user_insert_query);

    // query without parameters
    let post_insert_query = sqlx::query(
        "
        INSERT INTO post (post_id, user_id, content)
        VALUES
        ('252c1d98-a9b0-4f18-8298-e59058bdfe16', '6592b7c0-b531-4613-ace5-94246b7ce0c3', 'test post')
    ",
    );

    pipeline.push(post_insert_query);

    let comment_insert_query = sqlx::query(
        "
        INSERT INTO comment (comment_id, post_id, user_id, content)
        VALUES
        ($1, $2, $3, $4)
    ",
    )
    .bind(comment_id)
    .bind(post_id)
    .bind(user_id)
    .bind("test comment");

    pipeline.push(comment_insert_query);

    // 2. execute pipeline and validate PgQueryResult values
    let query_results = pipeline.execute(&pool).await?;

    for result in query_results {
        // each insert created a row
        assert_eq!(result.rows_affected(), 1);
    }

    // 3. assert the data was inserted
    ensure_test_data(false, user_id, post_id, comment_id, &pool).await?;

    // 4. cleanup
    cleanup_test_data(&pool, user_id, post_id, comment_id).await?;

    Ok(())
}
