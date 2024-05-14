mod common;

use common::TestDatabase;

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
        let db = TestDatabase::new("migrate_run_reversible_latest", "migrations_reversible");
        db.run_migration(false, None, false).success();
        assert_eq!(db.applied_migrations().await, all_migrations);
    }
    // With --target-version specified.
    {
        let db = TestDatabase::new(
            "migrate_run_reversible_latest_explicit",
            "migrations_reversible",
        );

        // Move to latest, explicitly specified.
        db.run_migration(false, Some(20230501000000), false)
            .success();
        assert_eq!(db.applied_migrations().await, all_migrations);

        // Move to latest when we're already at the latest.
        db.run_migration(false, Some(20230501000000), false)
            .success();
        assert_eq!(db.applied_migrations().await, all_migrations);

        // Upgrade to an old version.
        db.run_migration(false, Some(20230301000000), false)
            .failure();
        assert_eq!(db.applied_migrations().await, all_migrations);
    }
    // With --target-version, incrementally upgrade.
    {
        let db = TestDatabase::new(
            "migrate_run_reversible_incremental",
            "migrations_reversible",
        );

        // First version
        db.run_migration(false, Some(20230101000000), false)
            .success();
        assert_eq!(db.applied_migrations().await, vec![20230101000000]);

        // Dry run upgrade to latest.
        db.run_migration(false, None, true).success();
        assert_eq!(db.applied_migrations().await, vec![20230101000000]);

        // Dry run upgrade + 2
        db.run_migration(false, Some(20230301000000), true)
            .success();
        assert_eq!(db.applied_migrations().await, vec![20230101000000]);

        // Upgrade to non-existent version.
        db.run_migration(false, Some(20230901000000999), false)
            .failure();
        assert_eq!(db.applied_migrations().await, vec![20230101000000]);

        // Upgrade + 1
        db.run_migration(false, Some(20230201000000), false)
            .success();
        assert_eq!(
            db.applied_migrations().await,
            vec![20230101000000, 20230201000000]
        );

        // Upgrade + 2
        db.run_migration(false, Some(20230401000000), false)
            .success();
        assert_eq!(db.applied_migrations().await, all_migrations[..4]);
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
        let db = TestDatabase::new("migrate_revert_incremental", "migrations_reversible");
        db.run_migration(false, None, false).success();

        // Dry-run
        db.run_migration(true, None, true).success();
        assert_eq!(db.applied_migrations().await, all_migrations);

        // Downgrade one
        db.run_migration(true, None, false).success();
        assert_eq!(db.applied_migrations().await, all_migrations[..4]);

        // Downgrade one
        db.run_migration(true, None, false).success();
        assert_eq!(db.applied_migrations().await, all_migrations[..3]);
    }
    // With --target-version
    {
        let db = TestDatabase::new("migrate_revert_incremental", "migrations_reversible");
        db.run_migration(false, None, false).success();

        // Dry-run downgrade to version 3.
        db.run_migration(true, Some(20230301000000), true).success();
        assert_eq!(db.applied_migrations().await, all_migrations);

        // Downgrade to version 3.
        db.run_migration(true, Some(20230301000000), false)
            .success();
        assert_eq!(db.applied_migrations().await, all_migrations[..3]);

        // Try downgrading to the same version.
        db.run_migration(true, Some(20230301000000), false)
            .success();
        assert_eq!(db.applied_migrations().await, all_migrations[..3]);

        // Try downgrading to a newer version.
        db.run_migration(true, Some(20230401000000), false)
            .failure();
        assert_eq!(db.applied_migrations().await, all_migrations[..3]);

        // Try downgrading to a non-existent version.
        db.run_migration(true, Some(9999), false).failure();
        assert_eq!(db.applied_migrations().await, all_migrations[..3]);

        // Ensure we can still upgrade
        db.run_migration(false, Some(20230401000000), false)
            .success();
        assert_eq!(db.applied_migrations().await, all_migrations[..4]);

        // Downgrade to zero.
        db.run_migration(true, Some(0), false).success();
        assert_eq!(db.applied_migrations().await, vec![] as Vec<i64>);
    }
}
