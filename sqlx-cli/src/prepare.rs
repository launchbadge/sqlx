use anyhow::{bail, Context};
use console::style;
use remove_dir_all::remove_dir_all;
use serde::Deserialize;
use sqlx::any::{AnyConnectOptions, AnyKind};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::SystemTime;
use std::{env, fs};

type QueryData = BTreeMap<String, serde_json::Value>;
type JsonObject = serde_json::Map<String, serde_json::Value>;

pub fn run(url: &str, merge: bool, cargo_args: Vec<String>) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct DataFile {
        db: &'static str,
        #[serde(flatten)]
        data: QueryData,
    }

    anyhow::ensure!(
        Path::new("Cargo.toml").exists(),
        r#"Failed to read `Cargo.toml`.
hint: This command only works in the manifest directory of a Cargo package."#
    );

    let db_kind = get_db_kind(url)?;
    let data = run_prepare_step(merge, cargo_args)?;

    if data.is_empty() {
        println!(
            "{} no queries found; do you have the `offline` feature enabled in sqlx?",
            style("warning:").yellow()
        );
    }

    serde_json::to_writer_pretty(
        BufWriter::new(
            File::create("sqlx-data.json").context("failed to create/open `sqlx-data.json`")?,
        ),
        &DataFile { db: db_kind, data },
    )
    .context("failed to write to `sqlx-data.json`")?;

    println!(
        "query data written to `sqlx-data.json` in the current directory; \
         please check this into version control"
    );

    Ok(())
}

pub fn check(url: &str, merge: bool, cargo_args: Vec<String>) -> anyhow::Result<()> {
    let db_kind = get_db_kind(url)?;
    let data = run_prepare_step(merge, cargo_args)?;

    let data_file = File::open("sqlx-data.json").context(
        "failed to open `sqlx-data.json`; you may need to run `cargo sqlx prepare` first",
    )?;

    let mut saved_data: QueryData = serde_json::from_reader(BufReader::new(data_file))?;

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

fn run_prepare_step(merge: bool, cargo_args: Vec<String>) -> anyhow::Result<QueryData> {
    // path to the Cargo executable
    let cargo = env::var("CARGO")
        .context("`prepare` subcommand may only be invoked as `cargo sqlx prepare`")?;

    let output = Command::new(&cargo)
        .args(&["metadata", "--format-version=1"])
        .output()
        .context("Could not fetch metadata")?;

    #[derive(Deserialize)]
    struct Metadata {
        target_directory: PathBuf,
    }

    let metadata: Metadata =
        serde_json::from_slice(&output.stdout).context("Invalid `cargo metadata` output")?;

    // try removing the target/sqlx directory before running, as stale files
    // have repeatedly caused issues in the past.
    let _ = remove_dir_all(metadata.target_directory.join("sqlx"));

    let check_status = if merge {
        let check_status = Command::new(&cargo).arg("clean").status()?;

        if !check_status.success() {
            bail!("`cargo clean` failed with status: {}", check_status);
        }

        Command::new(&cargo)
            .arg("check")
            .args(cargo_args)
            .env(
                "RUSTFLAGS",
                format!(
                    "--cfg __sqlx_recompile_trigger=\"{}\"",
                    SystemTime::UNIX_EPOCH.elapsed()?.as_millis()
                ),
            )
            .env("SQLX_OFFLINE", "false")
            .status()?
    } else {
        Command::new(&cargo)
            .arg("rustc")
            .args(cargo_args)
            .arg("--")
            .arg("--emit")
            .arg("dep-info,metadata")
            // set an always-changing cfg so we can consistently trigger recompile
            .arg("--cfg")
            .arg(format!(
                "__sqlx_recompile_trigger=\"{}\"",
                SystemTime::UNIX_EPOCH.elapsed()?.as_millis()
            ))
            .env("SQLX_OFFLINE", "false")
            .status()?
    };

    if !check_status.success() {
        bail!("`cargo check` failed with status: {}", check_status);
    }

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

        // lazily remove the file, we don't care too much if we can't
        let _ = fs::remove_file(&path);
    }

    Ok(data)
}

fn get_db_kind(url: &str) -> anyhow::Result<&'static str> {
    let options = AnyConnectOptions::from_str(&url)?;

    // these should match the values of `DatabaseExt::NAME` in `sqlx-macros`
    match options.kind() {
        #[cfg(feature = "postgres")]
        AnyKind::Postgres => Ok("PostgreSQL"),

        #[cfg(feature = "mysql")]
        AnyKind::MySql => Ok("MySQL"),

        #[cfg(feature = "sqlite")]
        AnyKind::Sqlite => Ok("SQLite"),

        #[cfg(feature = "mssql")]
        AnyKind::Mssql => Ok("MSSQL"),
    }
}
