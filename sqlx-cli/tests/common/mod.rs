use assert_cmd::{assert::Assert, Command};

use sqlx::{
    migrate::{AppliedMigration, Migrate},
    Connection, SqliteConnection,
};
use std::{
    env::temp_dir,
    fs::remove_file,
    path::{Path, PathBuf},
};

pub struct TestDatabase {
    file_path: PathBuf,
}

impl TestDatabase {
    pub fn new(name: &str) -> Self {
        let file_path = Path::new(&temp_dir()).join(format!("test-{}.db", name));
        let ret = Self { file_path };
        Command::cargo_bin("cargo-sqlx")
            .unwrap()
            .args([
                "sqlx",
                "database",
                "create",
                "--database-url",
                &ret.connection_string(),
            ])
            .assert()
            .success();
        ret
    }

    pub fn run_migration(&self, source: &str, version: Option<i64>, dry_run: bool) -> Assert {
        let source_path = Path::new("tests").join(source);
        let source = String::from(source_path.to_str().unwrap());

        let ver = match version {
            Some(v) => v.to_string(),
            None => String::from(""),
        };
        Command::cargo_bin("cargo-sqlx")
            .unwrap()
            .args(
                [
                    vec![
                        "sqlx",
                        "migrate",
                        "run",
                        "--database-url",
                        &self.connection_string(),
                        "--source",
                        &source,
                    ],
                    match version {
                        Some(_) => vec!["--target-version", &ver],
                        None => vec![],
                    },
                    match dry_run {
                        true => vec!["--dry-run"],
                        false => vec![],
                    },
                ]
                .concat(),
            )
            .assert()
    }

    pub fn revert_migration(&self, source: &str, version: Option<i64>, dry_run: bool) -> Assert {
        let source_path = Path::new("tests").join(source);
        let source = String::from(source_path.to_str().unwrap());

        let ver = match version {
            Some(v) => v.to_string(),
            None => String::from(""),
        };
        Command::cargo_bin("cargo-sqlx")
            .unwrap()
            .args(
                [
                    vec![
                        "sqlx",
                        "migrate",
                        "revert",
                        "--database-url",
                        &self.connection_string(),
                        "--source",
                        &source,
                    ],
                    match version {
                        Some(_) => vec!["--target-version", &ver],
                        None => vec![],
                    },
                    match dry_run {
                        true => vec!["--dry-run"],
                        false => vec![],
                    },
                ]
                .concat(),
            )
            .assert()
    }

    pub async fn applied_migrations(&self) -> Vec<AppliedMigration> {
        let mut conn = SqliteConnection::connect(&self.connection_string())
            .await
            .unwrap();
        conn.list_applied_migrations().await.unwrap()
    }

    pub async fn applied_migrations_versions(&self) -> Vec<i64> {
        let mut conn = SqliteConnection::connect(&self.connection_string())
            .await
            .unwrap();
        conn.list_applied_migrations()
            .await
            .unwrap()
            .iter()
            .map(|m| m.version)
            .collect()
    }

    fn connection_string(&self) -> String {
        format!("sqlite://{}", self.file_path.display())
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        remove_file(&self.file_path).unwrap();
    }
}
