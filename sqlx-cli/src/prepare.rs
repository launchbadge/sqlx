use crate::opt::ConnectOpts;
use anyhow::{bail, Context};
use console::style;
use remove_dir_all::remove_dir_all;
use sqlx::any::{AnyConnectOptions, AnyKind};
use sqlx::Connection;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};

use crate::metadata::Metadata;

type QueryData = BTreeMap<String, serde_json::Value>;
type JsonObject = serde_json::Map<String, serde_json::Value>;

#[derive(serde::Serialize, serde::Deserialize)]
struct DataFile {
    db: String,
    #[serde(flatten)]
    data: QueryData,
}

pub async fn run(
    connect_opts: &ConnectOpts,
    merge: bool,
    cargo_args: Vec<String>,
) -> anyhow::Result<()> {
    // Ensure the database server is available.
    crate::connect(connect_opts).await?.close().await?;

    let url = &connect_opts.database_url;

    let db_kind = get_db_kind(url)?;
    let data = run_prepare_step(url, merge, cargo_args)?;

    if data.is_empty() {
        println!(
            "{} no queries found; please ensure that the `offline` feature is enabled in sqlx",
            style("warning:").yellow()
        );
    }

    serde_json::to_writer_pretty(
        BufWriter::new(
            File::create("sqlx-data.json").context("failed to create/open `sqlx-data.json`")?,
        ),
        &DataFile {
            db: db_kind.to_owned(),
            data,
        },
    )
    .context("failed to write to `sqlx-data.json`")?;

    println!(
        "query data written to `sqlx-data.json` in the current directory; \
         please check this into version control"
    );

    Ok(())
}

pub async fn check(
    connect_opts: &ConnectOpts,
    merge: bool,
    cargo_args: Vec<String>,
) -> anyhow::Result<()> {
    // Ensure the database server is available.
    crate::connect(connect_opts).await?.close().await?;

    let url = &connect_opts.database_url;

    let db_kind = get_db_kind(url)?;
    let data = run_prepare_step(url, merge, cargo_args)?;

    let data_file = File::open("sqlx-data.json").context(
        "failed to open `sqlx-data.json`; you may need to run `cargo sqlx prepare` first",
    )?;

    let DataFile {
        db: expected_db,
        data: saved_data,
    } = serde_json::from_reader(BufReader::new(data_file))?;

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

fn run_prepare_step(url: &str, merge: bool, cargo_args: Vec<String>) -> anyhow::Result<QueryData> {
    anyhow::ensure!(
        Path::new("Cargo.toml").exists(),
        r#"Failed to read `Cargo.toml`.
hint: This command only works in the manifest directory of a Cargo package."#
    );

    // path to the Cargo executable
    let cargo = env::var("CARGO")
        .context("`prepare` subcommand may only be invoked as `cargo sqlx prepare`")?;

    let output = Command::new(&cargo)
        .args(&["metadata", "--format-version=1"])
        .output()
        .context("Could not fetch metadata")?;

    let output_str =
        std::str::from_utf8(&output.stdout).context("Invalid `cargo metadata` output")?;
    let metadata: Metadata = output_str.parse()?;

    // try removing the target/sqlx directory before running, as stale files
    // have repeatedly caused issues in the past.
    let _ = remove_dir_all(metadata.target_directory().join("sqlx"));

    // Try only triggering a recompile on crates that use `sqlx-macros`, falling back to a full
    // clean on error.
    match setup_minimal_project_recompile(&cargo, &metadata, merge) {
        Ok(()) => {}
        Err(err) => {
            println!(
                "Failed minimal recompile setup. Cleaning entire project. Err: {}",
                err
            );
            let clean_status = Command::new(&cargo).arg("clean").status()?;
            if !clean_status.success() {
                bail!("`cargo clean` failed with status: {}", clean_status);
            }
        }
    };

    // Compile the queries.
    let check_status = {
        let mut check_command = Command::new(&cargo);
        check_command
            .arg("check")
            .args(cargo_args)
            .env("SQLX_OFFLINE", "false")
            .env("DATABASE_URL", url);

        // `cargo check` recompiles on changed rust flags which can be set either via the env var
        // or through the `rustflags` field in `$CARGO_HOME/config` when the env var isn't set.
        // Because of this we only pass in `$RUSTFLAGS` when present.
        if let Ok(rustflags) = env::var("RUSTFLAGS") {
            check_command.env("RUSTFLAGS", rustflags);
        }

        check_command.status()?
    };
    if !check_status.success() {
        bail!("`cargo check` failed with status: {}", check_status);
    }

    // Combine the queries into one file.
    let package_dir = if merge {
        // Merge queries from all workspace crates.
        "**"
    } else {
        // Use a separate sub-directory for each crate in a workspace. This avoids a race condition
        // where `prepare` can pull in queries from multiple crates if they happen to be generated
        // simultaneously (e.g. Rust Analyzer building in the background).
        metadata
            .current_package()
            .map(|pkg| pkg.name())
            .context("Resolving the crate package for the current working directory failed")?
    };
    let pattern = metadata
        .target_directory()
        .join("sqlx")
        .join(package_dir)
        .join("query-*.json");

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

#[derive(Debug, PartialEq)]
struct ProjectRecompileAction {
    // The names of the packages
    clean_packages: Vec<String>,
    touch_paths: Vec<PathBuf>,
}

/// Sets up recompiling only crates that depend on `sqlx-macros`
///
/// This gets a listing of all crates that depend on `sqlx-macros` (direct and transitive). The
/// crates within the current workspace have their source file's mtimes updated while crates
/// outside the workspace are selectively `cargo clean -p`ed. In this way we can trigger a
/// recompile of crates that may be using compile-time macros without forcing a full recompile.
///
/// If `workspace` is false, only the current package will have its files' mtimes updated.
fn setup_minimal_project_recompile(
    cargo: &str,
    metadata: &Metadata,
    workspace: bool,
) -> anyhow::Result<()> {
    let ProjectRecompileAction {
        clean_packages,
        touch_paths,
    } = if workspace {
        minimal_project_recompile_action(metadata)?
    } else {
        // Only touch the current crate.
        ProjectRecompileAction {
            clean_packages: Vec::new(),
            touch_paths: metadata.current_package().context("Failed to get package in current working directory, pass `--merged` if running from a workspace root")?.src_paths().to_vec(),
        }
    };

    for file in touch_paths {
        let now = filetime::FileTime::now();
        filetime::set_file_times(&file, now, now)
            .with_context(|| format!("Failed to update mtime for {:?}", file))?;
    }

    for pkg_id in &clean_packages {
        let clean_status = Command::new(cargo)
            .args(&["clean", "-p", pkg_id])
            .status()?;

        if !clean_status.success() {
            bail!("`cargo clean -p {}` failed", pkg_id);
        }
    }

    Ok(())
}

fn minimal_project_recompile_action(metadata: &Metadata) -> anyhow::Result<ProjectRecompileAction> {
    // Get all the packages that depend on `sqlx-macros`
    let mut sqlx_macros_dependents = BTreeSet::new();
    let sqlx_macros_ids: BTreeSet<_> = metadata
        .entries()
        // We match just by name instead of name and url because some people may have it installed
        // through different means like vendoring
        .filter(|(_, package)| package.name() == "sqlx-macros")
        .map(|(id, _)| id)
        .collect();
    for sqlx_macros_id in sqlx_macros_ids {
        sqlx_macros_dependents.extend(metadata.all_dependents_of(sqlx_macros_id));
    }

    // Figure out which `sqlx-macros` dependents are in the workspace vs out
    let mut in_workspace_dependents = Vec::new();
    let mut out_of_workspace_dependents = Vec::new();
    for dependent in sqlx_macros_dependents {
        if metadata.workspace_members().contains(&dependent) {
            in_workspace_dependents.push(dependent);
        } else {
            out_of_workspace_dependents.push(dependent);
        }
    }

    // In-workspace dependents have their source file's mtime updated. Out-of-workspace get
    // `cargo clean -p <PKGID>`ed
    let files_to_touch: Vec<_> = in_workspace_dependents
        .iter()
        .filter_map(|id| {
            metadata
                .package(id)
                .map(|package| package.src_paths().to_owned())
        })
        .flatten()
        .collect();
    let packages_to_clean: Vec<_> = out_of_workspace_dependents
        .iter()
        .filter_map(|id| {
            metadata
                .package(id)
                .map(|package| package.name().to_owned())
        })
        .collect();

    Ok(ProjectRecompileAction {
        clean_packages: packages_to_clean,
        touch_paths: files_to_touch,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::assert_eq;

    #[test]
    fn data_file_serialization_works() {
        let data_file = DataFile {
            db: "mysql".to_owned(),
            data: {
                let mut data = BTreeMap::new();
                data.insert("a".to_owned(), json!({"key1": "value1"}));
                data.insert("z".to_owned(), json!({"key2": "value2"}));
                data
            },
        };

        let data_file = serde_json::to_string(&data_file).expect("Data file serialized.");

        assert_eq!(
            data_file,
            "{\"db\":\"mysql\",\"a\":{\"key1\":\"value1\"},\"z\":{\"key2\":\"value2\"}}"
        );
    }

    #[test]
    fn data_file_deserialization_works() {
        let data_file =
            "{\"db\":\"mysql\",\"a\":{\"key1\":\"value1\"},\"z\":{\"key2\":\"value2\"}}";

        let data_file: DataFile = serde_json::from_str(data_file).expect("Data file deserialized.");
        let DataFile { db, data } = data_file;

        assert_eq!(db, "mysql");
        assert_eq!(data.len(), 2);
        assert_eq!(data.get("a"), Some(&json!({"key1": "value1"})));
        assert_eq!(data.get("z"), Some(&json!({"key2": "value2"})));
    }

    #[test]
    fn data_file_deserialization_works_for_ordered_keys() {
        let data_file =
            "{\"a\":{\"key1\":\"value1\"},\"db\":\"mysql\",\"z\":{\"key2\":\"value2\"}}";

        let data_file: DataFile = serde_json::from_str(data_file).expect("Data file deserialized.");
        let DataFile { db, data } = data_file;

        assert_eq!(db, "mysql");
        assert_eq!(data.len(), 2);
        assert_eq!(data.get("a"), Some(&json!({"key1": "value1"})));
        assert_eq!(data.get("z"), Some(&json!({"key2": "value2"})));
    }

    #[test]
    fn minimal_project_recompile_action_works() -> anyhow::Result<()> {
        let sample_metadata_path = Path::new("tests")
            .join("assets")
            .join("sample_metadata.json");
        let sample_metadata = std::fs::read_to_string(sample_metadata_path)?;
        let metadata: Metadata = sample_metadata.parse()?;

        let action = minimal_project_recompile_action(&metadata)?;
        assert_eq!(
            action,
            ProjectRecompileAction {
                clean_packages: vec!["sqlx".into()],
                touch_paths: vec![
                    "/home/user/problematic/workspace/b_in_workspace_lib/src/lib.rs".into(),
                    "/home/user/problematic/workspace/c_in_workspace_bin/src/main.rs".into(),
                ]
            }
        );

        Ok(())
    }
}
