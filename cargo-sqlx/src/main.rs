use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

use dotenv::dotenv;

use sqlx::PgConnection;
use sqlx::PgPool;

use structopt::StructOpt;

const MIGRATION_FOLDER: &'static str = "migrations";

/// Sqlx commandline tool
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx")]
enum Opt {
    // #[structopt(subcommand)]
    Migrate(MigrationCommand),
}

/// Simple postgres migrator
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx migrator")]
enum MigrationCommand {
    /// Initalizes new migration directory with db create script
    // Init {
    //     // #[structopt(long)]
    //     database_name: String,
    // },

    /// Add new migration with name <timestamp>_<migration_name>.sql
    Add {
        // #[structopt(long)]
        name: String,
    },

    /// Run all migrations
    Run,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    match opt {
        Opt::Migrate(command) => match command {
            // Opt::Init { database_name } => init_migrations(&database_name),
            MigrationCommand::Add { name } => add_migration_file(&name),
            MigrationCommand::Run => run_migrations().await,
        },
    }

    println!("All done!");
}

// fn init_migrations(db_name: &str) {
//     println!("Initing the migrations so hard! db: {:#?}", db_name);
// }

fn add_migration_file(name: &str) {
    use chrono::prelude::*;
    use std::path::Path;
    use std::path::PathBuf;

    if !Path::new(MIGRATION_FOLDER).exists() {
        fs::create_dir(MIGRATION_FOLDER).expect("Failed to create 'migrations' dir")
    }

    let dt = Utc::now();
    let mut file_name = dt.format("%Y-%m-%d_%H-%M-%S").to_string();
    file_name.push_str("_");
    file_name.push_str(name);
    file_name.push_str(".sql");

    let mut path = PathBuf::new();
    path.push(MIGRATION_FOLDER);
    path.push(&file_name);

    if path.exists() {
        eprintln!("Migration already exists!");
        return;
    }

    let mut file = File::create(path).expect("Failed to create file");
    file.write_all(b"-- Add migration script here")
        .expect("Could not write to file");

    println!("Created migration: '{}'", file_name);
}

pub struct Migration {
    pub name: String,
    pub sql: String,
}

fn load_migrations() -> Vec<Migration> {
    let entries = fs::read_dir(&MIGRATION_FOLDER).expect("Could not find 'migrations' dir");

    let mut migrations = Vec::new();

    for e in entries {
        if let Ok(e) = e {
            if let Ok(meta) = e.metadata() {
                if !meta.is_file() {
                    continue;
                }

                if let Some(ext) = e.path().extension() {
                    if ext != "sql" {
                        println!("Wrong ext: {:?}", ext);
                        continue;
                    }
                } else {
                    continue;
                }

                let mut file =
                    File::open(e.path()).expect(&format!("Failed to open: '{:?}'", e.file_name()));
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect(&format!("Failed to read: '{:?}'", e.file_name()));

                migrations.push(Migration {
                    name: e.file_name().to_str().unwrap().to_string(),
                    sql: contents,
                });
            }
        }
    }

    migrations.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

    migrations
}

async fn run_migrations() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("Failed to find 'DATABASE_URL'");

    let mut pool = PgPool::new(&db_url)
        .await
        .expect("Failed to connect to pool");

    create_migration_table(&mut pool).await;

    let migrations = load_migrations();

    for mig in migrations.iter() {
        let mut tx = pool.begin().await.unwrap();

        if check_if_applied(&mut tx, &mig.name).await {
            println!("Already applied migration: '{}'", mig.name);
            continue;
        }
        println!("Applying migration: '{}'", mig.name);

        sqlx::query(&mig.sql)
            .execute(&mut tx)
            .await
            .expect(&format!("Failed to run migration {:?}", &mig.name));

        save_applied_migration(&mut tx, &mig.name).await;

        tx.commit().await.unwrap();
    }
}

async fn create_migration_table(mut pool: &PgPool) {
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS __migrations (
    migration VARCHAR (255) PRIMARY KEY,
    created TIMESTAMP NOT NULL DEFAULT current_timestamp
);
    "#,
    )
    .execute(&mut pool)
    .await
    .expect("Failed to create migration table");
}

async fn check_if_applied(pool: &mut PgConnection, migration: &str) -> bool {
    use sqlx::postgres::PgRow;
    use sqlx::Row;

    sqlx::query("select exists(select migration from __migrations where migration = $1) as exists")
        .bind(migration.to_string())
        .try_map(|row: PgRow| row.try_get("exists"))
        .fetch_one(pool)
        .await
        .expect("Failed to check migration table")
}

async fn save_applied_migration(pool: &mut PgConnection, migration: &str) {
    sqlx::query("insert into __migrations (migration) values ($1)")
        .bind(migration.to_string())
        .execute(pool)
        .await
        .expect("Failed to insert migration ");
}
