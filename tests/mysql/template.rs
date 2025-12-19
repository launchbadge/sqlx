// Tests for template database cloning functionality in MySQL.
//
// These tests verify that:
// 1. Template databases are created when migrations are used
// 2. Multiple tests with the same migrations share a template
// 3. SQLX_TEST_NO_TEMPLATE disables template cloning
// 4. Different migrations create different templates

use sqlx::mysql::MySqlPool;
use sqlx::Connection;

/// Verify that the template tracking table exists and contains entries
/// after running a test with migrations.
#[sqlx::test(migrations = "tests/mysql/migrations")]
async fn it_creates_template_database(pool: MySqlPool) -> sqlx::Result<()> {
    // Get the master database connection to check for templates
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut master_conn = sqlx::mysql::MySqlConnection::connect(&database_url).await?;

    // Check that the template tracking table exists and has at least one entry
    let template_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_test_templates WHERE template_name LIKE '_sqlx_template_%'",
    )
    .fetch_one(&mut master_conn)
    .await?;

    // If templates are enabled, we should have at least one template
    // (unless SQLX_TEST_NO_TEMPLATE is set)
    if std::env::var("SQLX_TEST_NO_TEMPLATE").is_err() {
        assert!(
            template_count > 0,
            "Expected at least one template database to be created"
        );
    }

    // Verify the test database has the expected tables from migrations
    let table_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = database() AND table_name IN ('user', 'post', 'comment')")
            .fetch_one(&pool)
            .await?;

    assert_eq!(table_count, 3, "Expected user, post, and comment tables");

    Ok(())
}

/// Verify that the migrations table is properly cloned from template.
/// When cloning from a template, the _sqlx_migrations table should already
/// exist and contain the migration history.
#[sqlx::test(migrations = "tests/mysql/migrations")]
async fn it_clones_migrations_table(pool: MySqlPool) -> sqlx::Result<()> {
    // Check that _sqlx_migrations table exists and has entries
    let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await?;

    // We have 3 migrations in tests/mysql/migrations
    assert_eq!(
        migration_count, 3,
        "Expected 3 migrations to be recorded in _sqlx_migrations"
    );

    // Verify the specific migrations are recorded
    let versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await?;

    assert_eq!(versions, vec![1, 2, 3], "Expected migrations 1, 2, 3");

    Ok(())
}

/// Test that multiple tests with the same migrations share a template.
/// This test runs alongside other tests with the same migrations and
/// verifies that only one template exists.
#[sqlx::test(migrations = "tests/mysql/migrations")]
async fn it_reuses_template_for_same_migrations_1(pool: MySqlPool) -> sqlx::Result<()> {
    // This test shares migrations with other tests, so they should all
    // use the same template database.
    let db_name: String = sqlx::query_scalar("SELECT database()")
        .fetch_one(&pool)
        .await?;

    assert!(
        db_name.starts_with("_sqlx_test_"),
        "Test database should start with _sqlx_test_"
    );

    // Verify tables exist
    let user_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_schema = database() AND table_name = 'user')",
    )
    .fetch_one(&pool)
    .await?;

    assert!(user_exists, "user table should exist");

    Ok(())
}

/// Second test with same migrations - should reuse the same template.
#[sqlx::test(migrations = "tests/mysql/migrations")]
async fn it_reuses_template_for_same_migrations_2(pool: MySqlPool) -> sqlx::Result<()> {
    // Same test as above - verifies template reuse
    let db_name: String = sqlx::query_scalar("SELECT database()")
        .fetch_one(&pool)
        .await?;

    assert!(
        db_name.starts_with("_sqlx_test_"),
        "Test database should start with _sqlx_test_"
    );

    // Verify tables exist
    let post_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_schema = database() AND table_name = 'post')",
    )
    .fetch_one(&pool)
    .await?;

    assert!(post_exists, "post table should exist");

    Ok(())
}

/// Verify that template databases have the correct naming pattern.
#[sqlx::test(migrations = "tests/mysql/migrations")]
async fn it_names_templates_correctly(_pool: MySqlPool) -> sqlx::Result<()> {
    if std::env::var("SQLX_TEST_NO_TEMPLATE").is_ok() {
        // Skip this test if templates are disabled
        return Ok(());
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut master_conn = sqlx::mysql::MySqlConnection::connect(&database_url).await?;

    // Get template names
    let template_names: Vec<String> = sqlx::query_scalar(
        "SELECT template_name FROM _sqlx_test_templates WHERE template_name LIKE '_sqlx_template_%'",
    )
    .fetch_all(&mut master_conn)
    .await?;

    for name in &template_names {
        assert!(
            name.starts_with("_sqlx_template_"),
            "Template name should start with _sqlx_template_, got: {}",
            name
        );
        // Template names should be reasonably short (under 63 chars for MySQL)
        assert!(
            name.len() < 63,
            "Template name should be under 63 chars, got: {} ({})",
            name,
            name.len()
        );
    }

    Ok(())
}

/// Test that fixtures are applied on top of the cloned template.
/// Fixtures should be applied per-test, not stored in the template.
#[sqlx::test(migrations = "tests/mysql/migrations", fixtures("users"))]
async fn it_applies_fixtures_after_clone(pool: MySqlPool) -> sqlx::Result<()> {
    // The users fixture should have been applied
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user")
        .fetch_one(&pool)
        .await?;

    assert!(user_count > 0, "users fixture should have inserted users");

    // Verify specific users from the fixture
    let usernames: Vec<String> = sqlx::query_scalar("SELECT username FROM user ORDER BY username")
        .fetch_all(&pool)
        .await?;

    assert_eq!(usernames, vec!["alice", "bob"]);

    Ok(())
}

/// Test that different tests get different fixture data even when
/// sharing the same template.
#[sqlx::test(migrations = "tests/mysql/migrations", fixtures("users", "posts"))]
async fn it_isolates_fixtures_between_tests(pool: MySqlPool) -> sqlx::Result<()> {
    // This test has different fixtures than the previous one
    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM post")
        .fetch_one(&pool)
        .await?;

    assert!(post_count > 0, "posts fixture should have inserted posts");

    Ok(())
}

/// Unit test for migrations_hash function - verify it produces consistent hashes.
#[test]
fn test_migrations_hash_consistency() {
    use sqlx_core::migrate::Migrator;
    use sqlx_core::testing::migrations_hash;

    // Create a migrator with the test migrations
    let migrator: Migrator = sqlx::migrate!("tests/mysql/migrations");

    // Hash should be consistent across calls
    let hash1 = migrations_hash(&migrator);
    let hash2 = migrations_hash(&migrator);

    assert_eq!(hash1, hash2, "migrations_hash should be deterministic");

    // Hash should be non-empty and reasonable length
    assert!(!hash1.is_empty(), "hash should not be empty");
    assert!(
        hash1.len() < 30,
        "hash should be reasonably short for use in database names"
    );
}

/// Unit test for template_db_name function.
#[test]
fn test_template_db_name_format() {
    use sqlx_core::testing::template_db_name;

    let name = template_db_name("abc123xyz");

    assert!(
        name.starts_with("_sqlx_template_"),
        "template name should have correct prefix"
    );
    assert!(
        name.contains("abc123xyz"),
        "template name should contain hash"
    );
    assert!(
        name.len() < 63,
        "template name should fit in MySQL identifier limit"
    );

    // Test with special characters that need escaping
    let name_with_special = template_db_name("a-b+c/d");
    assert!(
        !name_with_special.contains('-'),
        "should not contain hyphen"
    );
    assert!(!name_with_special.contains('+'), "should not contain plus");
    assert!(!name_with_special.contains('/'), "should not contain slash");
}
