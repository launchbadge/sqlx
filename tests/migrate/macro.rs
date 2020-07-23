use sqlx::migrate::Migrator;
use std::path::Path;

#[sqlx_macros::test]
async fn same_output() -> anyhow::Result<()> {
    let embedded = sqlx::migrate!("tests/migrate/migrations");
    let runtime = Migrator::new(Path::new("tests/migrate/migrations")).await?;

    for (e, r) in embedded.iter().zip(runtime.iter()) {
        assert_eq!(e.version, r.version);
        assert_eq!(e.description, r.description);
        assert_eq!(e.sql, r.sql);
        assert_eq!(e.checksum, r.checksum);
    }

    Ok(())
}
