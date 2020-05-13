use anyhow::{anyhow, bail, Context};
use std::process::Command;
use std::{env, fs};

use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;
use url::Url;

type QueryData = BTreeMap<String, serde_json::Value>;
type JsonObject = serde_json::Map<String, serde_json::Value>;

pub fn run() -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct DataFile {
        db: &'static str,
        #[serde(flatten)]
        data: QueryData,
    }

    let db_kind = get_db_kind()?;
    let data = run_prepare_step()?;

    serde_json::to_writer_pretty(
        File::create("sqlx-data.json")?,
        &DataFile { db: db_kind, data },
    )
    .map_err(Into::into)
}

pub fn check() -> anyhow::Result<()> {
    let db_kind = get_db_kind()?;
    let data = run_prepare_step()?;

    let data_file = fs::read("sqlx-data.json").context(
        "failed to open `sqlx-data.json`; you may need to run `cargo sqlx prepare` first",
    )?;

    let mut saved_data: QueryData = serde_json::from_slice(&data_file)?;

    let expected_db = saved_data
        .remove("db")
        .context("expected key `db` in data file")?;

    let expected_db = expected_db
        .as_str()
        .context("expected key `db` to be a string")?;

    if db_kind != expected_db {
        bail!(
            "saved prepare data is for {}, not {} (inferred from `DATABASE_URL`)",
            expected_db,
            db_kind
        )
    }

    if data != saved_data {
        bail!("`cargo sqlx prepare` needs to be rerun")
    }

    Ok(())
}

fn run_prepare_step() -> anyhow::Result<QueryData> {
    // path to the Cargo executable
    let cargo = env::var("CARGO")
        .context("`prepare` subcommand may only be invoked as `cargo sqlx prepare``")?;

    if !Command::new(cargo).arg("check").status()?.success() {
        bail!("`cargo check` failed");
    }

    let save_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target/sqlx".into());
    let pattern = Path::new(&save_dir).join("/query-*.json");

    let mut data = BTreeMap::new();

    for path in glob::glob(
        pattern
            .to_str()
            .context("CARGO_TARGET_DIR not valid UTF-8")?,
    )? {
        let path = path?;
        let contents = fs::read(&*path)?;
        let mut query_data: JsonObject = serde_json::from_slice(&contents)?;

        // we lift the `hash` key to the outer map
        let hash = query_data
            .remove("hash")
            .context("expected key `hash` in query data")?;

        if let serde_json::Value::String(hash) = hash {
            data.insert(hash, serde_json::Value::Object(query_data));
        } else {
            bail!(
                "expected key `hash` in query data to be string, was {:?} instead; file: {}",
                hash,
                path.display()
            )
        }
    }

    Ok(data)
}

fn get_db_kind() -> anyhow::Result<&'static str> {
    let db_url = dotenv::var("DATABASE_URL")
        .map_err(|_| anyhow!("`DATABASE_URL` must be set to use the `prepare` subcommand"))?;

    let db_url = Url::parse(&db_url)?;

    // these should match the values of `DatabaseExt::NAME` in `sqlx-macros`
    match db_url.scheme() {
        "postgres" | "postgresql" => Ok("PostgreSQL"),
        "mysql" | "mariadb" => Ok("MySQL/MariaDB"),
        "sqlite" => Ok("SQLite"),
        _ => bail!("unexpected scheme in database URL: {}", db_url.scheme()),
    }
}
