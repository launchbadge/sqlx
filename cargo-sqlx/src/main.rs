use sha3::{Digest, Sha3_512};
use sqlx::PgPool;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use structopt::StructOpt;

#[macro_use]
extern crate anyhow;

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo-sqlx", about = "SQLx migration tool")]
enum Opt {
    New { migration: String },
    Run,
}

#[derive(Debug)]
struct Migration {
    migration: OsString,
    hash: Vec<u8>,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let migrations = std::env::var("SQLX_MIGRATIONS_DIR").unwrap_or_else(|_| "./migrations".to_owned());

    let path = Path::new(&migrations);

    // If let chains WHEN???
    if let Ok(metadata) = fs::metadata(path) {
        if !metadata.is_dir() {
            return Err(anyhow!(
                "Migrations directory is not a directoy as expected"
            ));
        }
    } else {
        fs::create_dir(path).unwrap();
    }

    match opt {
        Opt::New { migration } => new(path, migration),
        Opt::Run => run(path).await,
    }?;

    Ok(())
}

fn new<T: AsRef<Path>>(path: T, migration: String) -> Result<(), anyhow::Error> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let path = path.as_ref().join(format!("{}-{}.sql", time, migration));

    fs::File::create(path)?;

    Ok(())
}

async fn run<T: AsRef<Path>>(path: T) -> Result<(), anyhow::Error> {
    let database = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow!("DATABASE_URL environment variable MUST be set"))?;

    let mut pool = PgPool::new(&database).await?;

    sqlx::query("create table if not exists sqlx_migrations (migration text not null primary key, hash bytea not null)").execute(&mut pool).await?;

    let mut dirs = fs::read_dir(path)?
        .filter_map(Result::ok)
        .collect::<Vec<fs::DirEntry>>();
    dirs.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    let migrations = sqlx::query!("select * from sqlx_migrations order by migration asc")
        .fetch_all(&mut pool)
        .await?
        .into_iter()
        .map(|m| Migration {
            migration: OsString::from(m.migration),
            hash: m.hash,
        })
        .collect::<Vec<Migration>>();

    for (index, _) in dirs.iter().enumerate() {
        let filename = dirs[index].file_name().into_string().unwrap();

        if let Ok(migration_to_run) = fs::read_to_string(dirs[index].path()) {
            if migration_to_run != "" {
                let mut hasher = Sha3_512::new();
                hasher.input(&migration_to_run.as_bytes());
                let hash = hasher.result().as_slice().to_vec();

                if let Some(upstream_migration) = migrations.get(index) {
                    if std::cmp::Ordering::Equal != hash.cmp(&upstream_migration.hash) {
                        return Err(anyhow!("{:?} is not synced with the database.", filename));
                    }
                } else {
                    sqlx::query(&migration_to_run).execute(&mut pool).await?;

                    sqlx::query("INSERT INTO sqlx_migrations (migration, hash) VALUES ($1, $2)")
                        .bind(filename)
                        .bind(hash)
                        .execute(&mut pool)
                        .await?;
                }
            }
        }
    }

    Ok(())
}
