use std::fs;
use std::path::Path;
use structopt::StructOpt;

mod new;
mod run;

#[macro_use]
extern crate anyhow;

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo-sqlx", about = "SQLx migration tool")]
enum Opt {
    New { migration: String },
    Run,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let migrations =
        std::env::var("SQLX_MIGRATIONS_DIR").unwrap_or_else(|_| "./migrations".to_owned());

    let path = Path::new(&migrations);

    // If let chains WHEN???
    if let Ok(metadata) = fs::metadata(path) {
        if !metadata.is_dir() {
            return Err(anyhow!(
                "Migrations directory is not a directoy as expected"
            ));
        }
    } else {
        fs::create_dir_all(path).unwrap();
    }

    match opt {
        Opt::New { migration } => new::new(path, migration),
        Opt::Run => run::run(path).await,
    }?;

    Ok(())
}
