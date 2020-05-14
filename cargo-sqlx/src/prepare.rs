use anyhow::{anyhow, bail, Context};
use std::process::Command;
use std::{env, fs};

use cargo_metadata::MetadataCommand;
use std::collections::BTreeMap;
use std::fs::File;

use std::time::SystemTime;
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
        File::create("sqlx-data.json").context("failed to create/open `sqlx-data.json`")?,
        &DataFile { db: db_kind, data },
    )
    .context("failed to write to `sqlx-data.json`")?;

    println!(
        "query data written to `sqlx-data.json` in the current directory; \
         please check this into version control"
    );

    Ok(())
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

    let check_status = Command::new(&cargo)
        .arg("check")
        // set an always-changing env var that the macros depend on via `env!()`
        .env(
            "__SQLX_RECOMPILE_TRIGGER",
            SystemTime::UNIX_EPOCH.elapsed()?.as_millis().to_string(),
        )
        .status()?;

    if !check_status.success() {
        bail!("`cargo check` failed with status: {}", check_status);
    }

    let metadata = MetadataCommand::new()
        .cargo_path(cargo)
        .exec()
        .context("failed to execute `cargo metadata`")?;

    let pattern = metadata.target_directory.join("sqlx/query-*.json");

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
