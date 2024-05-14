use assert_cmd::{assert::Assert, Command};

use sqlx::{migrate::Migrate, Connection, SqliteConnection};
use std::{
    env::temp_dir,
    fs::remove_file,
    path::{Path, PathBuf},
};

pub struct TestDatabase {
    file_path: PathBuf,
    migrations: String,
}

impl TestDatabase {
    pub fn new(name: &str, migrations: &str) -> Self {
        let migrations_path = Path::new("tests").join(migrations);
        let file_path = Path::new(&temp_dir()).join(format!("test-{}.db", name));
        let ret = Self {
            file_path,
            migrations: String::from(migrations_path.to_str().unwrap()),
        };
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

    pub fn connection_string(&self) -> String {
        format!("sqlite://{}", self.file_path.display())
    }

    pub fn run_migration(&self, revert: bool, version: Option<i64>, dry_run: bool) -> Assert {
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
                        match revert {
                            true => "revert",
                            false => "run",
                        },
                        "--database-url",
                        &self.connection_string(),
                        "--source",
                        &self.migrations,
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

    pub async fn applied_migrations(&self) -> Vec<i64> {
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
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        remove_file(&self.file_path).unwrap();
    }
}
