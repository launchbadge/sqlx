mod common;

use common::TestDatabase;
use sqlx::migrate::AppliedMigration;

#[tokio::test]
async fn run_reversible_migrations() {
    let all_migrations: Vec<i64> = vec![
        20230101000000,
        20230201000000,
        20230301000000,
        20230401000000,
        20230501000000,
    ];
    // Without --target-version specified.k
    {
        let db = TestDatabase::new("migrate_run_reversible_latest");
        db.run_migration("migrations_reversible", None, false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations);
    }
    // With --target-version specified.
    {
        let db = TestDatabase::new("migrate_run_reversible_latest_explicit");

        // Move to latest, explicitly specified.
        db.run_migration("migrations_reversible", Some(20230501000000), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations);

        // Move to latest when we're already at the latest.
        db.run_migration("migrations_reversible", Some(20230501000000), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations);

        // Upgrade to an old version.
        db.run_migration("migrations_reversible", Some(20230301000000), false)
            .failure();
        assert_eq!(db.applied_migrations_versions().await, all_migrations);
    }
    // With --target-version, incrementally upgrade.
    {
        let db = TestDatabase::new("migrate_run_reversible_incremental");

        // First version
        db.run_migration("migrations_reversible", Some(20230101000000), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, vec![20230101000000]);

        // Dry run upgrade to latest.
        db.run_migration("migrations_reversible", None, true)
            .success();
        assert_eq!(db.applied_migrations_versions().await, vec![20230101000000]);

        // Dry run upgrade + 2
        db.run_migration("migrations_reversible", Some(20230301000000), true)
            .success();
        assert_eq!(db.applied_migrations_versions().await, vec![20230101000000]);

        // Upgrade to non-existent version.
        db.run_migration("migrations_reversible", Some(20230901000000999), false)
            .failure();
        assert_eq!(db.applied_migrations_versions().await, vec![20230101000000]);

        // Upgrade + 1
        db.run_migration("migrations_reversible", Some(20230201000000), false)
            .success();
        assert_eq!(
            db.applied_migrations_versions().await,
            vec![20230101000000, 20230201000000]
        );

        // Upgrade + 2
        db.run_migration("migrations_reversible", Some(20230401000000), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..4]);
    }
}

#[tokio::test]
async fn revert_migrations() {
    let all_migrations: Vec<i64> = vec![
        20230101000000,
        20230201000000,
        20230301000000,
        20230401000000,
        20230501000000,
    ];

    // Without --target-version
    {
        let db = TestDatabase::new("migrate_revert_incremental");
        db.run_migration("migrations_reversible", None, false)
            .success();

        // Dry-run
        db.revert_migration("migrations_reversible", None, true)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations);

        // Downgrade one
        db.revert_migration("migrations_reversible", None, false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..4]);

        // Downgrade one
        db.revert_migration("migrations_reversible", None, false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..3]);
    }
    // With --target-version
    {
        let db = TestDatabase::new("migrate_revert_incremental");
        db.run_migration("migrations_reversible", None, false)
            .success();

        // Dry-run downgrade to version 3.
        db.revert_migration("migrations_reversible", Some(20230301000000), true)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations);

        // Downgrade to version 3.
        db.revert_migration("migrations_reversible", Some(20230301000000), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..3]);

        // Try downgrading to the same version.
        db.revert_migration("migrations_reversible", Some(20230301000000), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..3]);

        // Try downgrading to a newer version.
        db.revert_migration("migrations_reversible", Some(20230401000000), false)
            .failure();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..3]);

        // Try downgrading to a non-existent version.
        db.revert_migration("migrations_reversible", Some(9999), false)
            .failure();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..3]);

        // Ensure we can still upgrade
        db.run_migration("migrations_reversible", Some(20230401000000), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, all_migrations[..4]);

        // Downgrade to zero.
        db.revert_migration("migrations_reversible", Some(0), false)
            .success();
        assert_eq!(db.applied_migrations_versions().await, vec![] as Vec<i64>);
    }
}

#[tokio::test]
async fn run_on_change_migrations() {
    // Given
    let db = TestDatabase::new("migrate_on_changes");

    // When
    db.run_migration("migrations_on_change/original_set", None, false)
        .success();

    // Then
    let applied_migrations = db.applied_migrations_versions().await;
    assert_eq!(2, applied_migrations.len());
}

#[tokio::test]
async fn run_on_change_migrations_twice() {
    // Given
    let db = TestDatabase::new("migrate_on_changes_twice");

    // When
    db.run_migration("migrations_on_change/original_set", None, false)
        .success();

    // Then
    let applied_migrations = db.applied_migrations().await;

    // Re-run same migrations without changes
    db.run_migration("migrations_on_change/original_set", None, false)
        .success();

    let reapplied_migrations = db.applied_migrations().await;
    assert_eq!(2, reapplied_migrations.len());
    assert_eq!(applied_migrations, reapplied_migrations)
}

#[tokio::test]
async fn run_updated_on_change_migrations_() {
    // Given
    let db = TestDatabase::new("migrate_updated_on_changes");

    // When
    db.run_migration("migrations_on_change/original_set", None, false)
        .success();

    // Then
    let applied_migrations = db.applied_migrations().await;

    // Re-run same migrations with 'some_script.onchange.sql' updated
    db.run_migration("migrations_on_change/with_first_updated", None, false)
        .success();

    let reapplied_migrations = db.applied_migrations().await;
    assert_eq!(2, reapplied_migrations.len());

    // Assert that first script has been rerun and updated
    assert_ne_checksums(
        "first script".to_string(),
        &applied_migrations,
        &reapplied_migrations,
    );

    // Assert that second script has NOT been rerun
    assert_eq_migrations(
        "second script".to_string(),
        &applied_migrations,
        &reapplied_migrations,
    );
}

#[tokio::test]
async fn run_on_change_migrations_with_missing_ones() {
    // Given
    let db = TestDatabase::new("migrate_on_changes_with_missing_ones");

    // When
    db.run_migration("migrations_on_change/original_set", None, false)
        .success();

    // Then
    let applied_migrations = db.applied_migrations().await;
    assert_eq!(2, applied_migrations.len());

    // Re-run same migrations without changes
    db.run_migration(
        "migrations_on_change/with_one_deleted_and_one_added",
        None,
        false,
    )
    .success();

    let reapplied_migrations = db.applied_migrations().await;
    assert_eq!(3, reapplied_migrations.len());

    assert_eq_migrations(
        "second script".to_string(),
        &applied_migrations,
        &reapplied_migrations,
    );
}

#[tokio::test]
async fn run_on_change_migrations_with_classic_ones() {
    // Given
    let db = TestDatabase::new("migrate_on_changes_with_classic_ones");

    // When
    db.run_migration("migrations_on_change/original_set", None, false)
        .success();

    // Then
    let applied_migrations = db.applied_migrations().await;
    assert_eq!(2, applied_migrations.len());

    // Re-run same migrations without changes
    db.run_migration("migrations_on_change/with_classic_ones", None, false)
        .success();

    let reapplied_migrations = db.applied_migrations().await;
    assert_eq!(4, reapplied_migrations.len());

    assert_eq_migrations(
        "first script".to_string(),
        &applied_migrations,
        &reapplied_migrations,
    );

    assert_eq_migrations(
        "second script".to_string(),
        &applied_migrations,
        &reapplied_migrations,
    );
}

#[tokio::test]
async fn run_on_change_migrations_with_brand_one() {
    // Given
    let db = TestDatabase::new("migrate_on_changes_with_brand_one");

    // When
    db.run_migration("migrations_on_change/original_set", None, false)
        .success();

    // Then
    let applied_migrations = db.applied_migrations().await;
    assert_eq!(2, applied_migrations.len());

    // Re-run same migrations without changes
    db.run_migration("migrations_on_change/with_brand_one", None, false)
        .success();

    let reapplied_migrations = db.applied_migrations().await;
    assert_eq!(3, reapplied_migrations.len());

    assert_eq_migrations(
        "first script".to_string(),
        &applied_migrations,
        &reapplied_migrations,
    );

    assert_eq_migrations(
        "second script".to_string(),
        &applied_migrations,
        &reapplied_migrations,
    );
}

fn assert_eq_migrations(
    migration_description: String,
    first_applied: &Vec<AppliedMigration>,
    second_applied: &Vec<AppliedMigration>,
) {
    let first_applied = first_applied
        .iter()
        .find(|&am| am.description == migration_description)
        .unwrap();

    let second_applied = second_applied
        .iter()
        .find(|&am| am.description == migration_description)
        .unwrap();

    assert_eq!(first_applied, second_applied);
}

fn assert_ne_checksums(
    migration_description: String,
    first_applied: &Vec<AppliedMigration>,
    second_applied: &Vec<AppliedMigration>,
) {
    let first_applied = first_applied
        .iter()
        .find(|&am| am.description == migration_description)
        .unwrap();

    let second_applied = second_applied
        .iter()
        .find(|&am| am.description == migration_description)
        .unwrap();

    assert_ne!(first_applied.checksum, second_applied.checksum);
}
