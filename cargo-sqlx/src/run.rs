use sha3::{Digest, Sha3_512};
use sqlx::mysql::MySqlRow;
use sqlx::postgres::PgRow;
use sqlx::row::Row;
use sqlx::FromRow;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use sqlx::postgres::PgConnection;
use sqlx::mysql::MySqlConnection;
use sqlx::Connect;
use sqlx::Connection;
use sqlx::Database;
use sqlx::encode::Encode;

#[derive(Debug)]
struct Migration {
    migration: OsString,
    hash: Vec<u8>,
}

impl FromRow<PgRow> for Migration {
    fn from_row(row: PgRow) -> Migration {
        let migration = row.get::<String, _>("migration");
        let hash = row.get::<Vec<u8>, _>("hash");
        Migration {
            migration: OsString::from(migration),
            hash,
        }
    }
}

impl FromRow<MySqlRow> for Migration {
    fn from_row(row: MySqlRow) -> Migration {
        let migration = row.get::<String, _>("migration");
        let hash = row.get::<Vec<u8>, _>("hash");
        Migration {
            migration: OsString::from(migration),
            hash,
        }
    }
}

pub async fn run<T: AsRef<Path>>(path: T) -> Result<(), anyhow::Error> {
    let database = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow!("DATABASE_URL environment variable MUST be set"))?;

    match database.as_str() {
        "postgressql" | "postgres" => blah(PgConnection::connect(&database).await?, path).await,
        "mysql" | "mariadb" => blah(MySqlConnection::connect(&database).await?, path).await,
        _ => Err(anyhow!("Unsupported database")),
    }
}

trait Hash {
    fn hash(&self) -> Vec<u8>;
}

impl Hash for String {
    fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_512::new();
        hasher.input(&self.as_bytes());
        hasher.result().as_slice().to_vec()
    }
}

async fn blah<C, P>(mut conn: C, path: P) -> anyhow::Result<()> 
where 
    C: Connection,
    <C as sqlx::Executor>::Database: Sized,
    <C as sqlx::Executor>::Database: sqlx::types::HasSqlType<Vec<u8>>,
    <C as sqlx::Executor>::Database: sqlx::types::HasSqlType<String>,
    Vec<u8>: Encode<<C as sqlx::Executor>::Database>,
    String: Encode<<C as sqlx::Executor>::Database>,
    Migration: FromRow<<<C as sqlx::Executor>::Database as Database>::Row>,
    P: AsRef<Path>, {
    sqlx::query("create table if not exists sqlx_migrations (migration text not null primary key, hash bytea not null)").execute(&mut conn).await?;

    let mut files = fs::read_dir(path)?
        .filter_map(Result::ok)
        .collect::<Vec<fs::DirEntry>>();
    files.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    let migrations =
        sqlx::query_as::<<C as sqlx::Executor>::Database, Migration>("select * from sqlx_migrations order by migration asc")
            .fetch_all(&mut conn)
            .await?;

    for (index, file) in files.iter().enumerate() {
        let filename = file.file_name().into_string().unwrap();

        if let Ok(migration_to_run) = fs::read_to_string(file.path()) {
            if migration_to_run != "" {
                let hash = migration_to_run.hash();
                if let Some(upstream_migration) = migrations.get(index) {
                    if std::cmp::Ordering::Equal != hash.cmp(&upstream_migration.hash) {
                        return Err(anyhow!("{:?} is not synced with the database.", filename));
                    }
                } else {
                    sqlx::query(&migration_to_run).execute(&mut conn).await?;

                    sqlx::query("INSERT INTO sqlx_migrations (migration, hash) VALUES ($1, $2)")
                        .bind(filename)
                        .bind(hash)
                        .execute(&mut conn)
                        .await?;
                }
            }
        }
    }

    Ok(())
}
