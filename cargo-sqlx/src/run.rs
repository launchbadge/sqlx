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
use url::Url;
use std::io;

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

    let url = Url::parse(&database)?;

    match url.scheme() {
        "postgressql" | "postgres" => migrate(PgConnection::connect(&database).await?, path).await,
        "mysql" | "mariadb" => migrate(MySqlConnection::connect(&database).await?, path).await,
        _ => Err(anyhow!("Unsupported database")),
    }
}

trait Hash {
    type Target;
    fn hash(&self) -> Self::Target;
}

impl Hash for String {
    type Target = Vec<u8>;

    fn hash(&self) -> Self::Target {
        let mut hasher = Sha3_512::new();
        hasher.input(&self.as_bytes());
        hasher.result().as_slice().to_vec()
    }
}

impl Hash for io::Result<String> {
    type Target = Option<Vec<u8>>;

    fn hash(&self) -> Self::Target {
        match self {
            Ok(value) => Some(value.hash()),
            _ => None
        }
    }
}

impl<'a> Hash for Option<&'a Migration> {
    type Target = Option<&'a Vec<u8>>;

    fn hash(&self) -> Option<&'a Vec<u8>> {
        if let Some(value) = self {
            Some(&value.hash)
        } else {
            None
        }
    }
}

macro_rules! conditional_rollback {
    ($conn: ident, $ident: ident) => {
        match $ident {
            Err(err) => {
                $conn.send("rollback").await?;
                return Err(err)?;
            },
            Ok(value) => value,
        };
    }
}

async fn migrate<C, P>(mut conn: C, path: P) -> anyhow::Result<()> 
where 
    C: Connection,
    <C as sqlx::Executor>::Database: Sized,
    <C as sqlx::Executor>::Database: sqlx::types::HasSqlType<Vec<u8>>,
    <C as sqlx::Executor>::Database: sqlx::types::HasSqlType<String>,
    Vec<u8>: Encode<<C as sqlx::Executor>::Database>,
    String: Encode<<C as sqlx::Executor>::Database>,
    Migration: FromRow<<<C as sqlx::Executor>::Database as Database>::Row>,
    P: AsRef<Path>, {
    conn.send("begin").await?;

    let result = conn.send("create table if not exists sqlx_migrations (migration text not null primary key, hash bytea not null)").await;
    conditional_rollback!(conn, result);

    let mut files = fs::read_dir(path)?
        .filter_map(Result::ok)
        .collect::<Vec<fs::DirEntry>>();
    files.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    let migrations =
        sqlx::query_as::<<C as sqlx::Executor>::Database, Migration>("select * from sqlx_migrations order by migration asc")
            .fetch_all(&mut conn)
            .await;
    let migrations = conditional_rollback!(conn, migrations);

    for (index, file) in files.iter().enumerate() {
        let filename = file.file_name().into_string().unwrap();

        let migration = fs::read_to_string(file.path());
        match (&migration, &migration.hash(), migrations.get(index).hash()) {
            (Ok(ref migration_to_run), Some(migration_to_run_hash), Some(upstream_migration_hash)) if migration_to_run_hash == upstream_migration_hash => {
                let result = conn.send(migration_to_run).await;
                conditional_rollback!(conn, result);

                let result = sqlx::query("INSERT INTO sqlx_migrations (migration, hash) VALUES ($1, $2)")
                    .bind(filename)
                    .bind(migration_to_run_hash)
                    .execute(&mut conn)
                    .await;
                conditional_rollback!(conn, result);
            }

            (_, _, _) => {
                conn.send("rollback").await?;
                return Err(anyhow!("{:?} is not synced with the database.", filename));
            }
        }
    }

    let result = conn.send("commit").await;
    conditional_rollback!(conn, result);

    Ok(())
}
