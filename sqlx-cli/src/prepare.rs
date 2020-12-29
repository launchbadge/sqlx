use anyhow::bail;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug)]
pub struct PrepareCtx {
    pub workspace: bool,
    pub cargo: OsString,
    pub cargo_args: Vec<String>,
    pub manifest_dir: PathBuf,
    pub target_dir: PathBuf,
    pub workspace_root: PathBuf,
    pub database_url: Option<String>,
}

pub fn run(ctx: &PrepareCtx) -> anyhow::Result<()> {
    let root = if ctx.workspace {
        &ctx.workspace_root
    } else {
        &ctx.manifest_dir
    };

    run_prepare_step(ctx, &root.join(".sqlx"))?;

    println!(
        "query data written to `.sqlx` in the current directory; \
         please check this into version control"
    );

    Ok(())
}

pub fn check(ctx: &PrepareCtx) -> anyhow::Result<()> {
    let cache_dir = ctx.target_dir.join("sqlx");
    run_prepare_step(ctx, &cache_dir)?;

    // TODO: Compare .sqlx to target/sqlx
    // * For files thta are only in the former, raise a warning
    // * For files that are only in the latter, raise an error

    Ok(())
}

fn run_prepare_step(ctx: &PrepareCtx, cache_dir: &Path) -> anyhow::Result<()> {
    anyhow::ensure!(
        Path::new("Cargo.toml").exists(),
        r#"Failed to read `Cargo.toml`.
hint: This command only works in the manifest directory of a Cargo package."#
    );

    if cache_dir.exists() {
        clear_cache_dir(cache_dir)?;
    } else {
        fs::create_dir(cache_dir)?;
    }

    let mut check_cmd = Command::new(&ctx.cargo);
    if ctx.workspace {
        let check_status = Command::new(&ctx.cargo).arg("clean").status()?;

        if !check_status.success() {
            bail!("`cargo clean` failed with status: {}", check_status);
        }

        check_cmd.arg("check").args(&ctx.cargo_args).env(
            "RUSTFLAGS",
            format!(
                "--cfg __sqlx_recompile_trigger=\"{}\"",
                SystemTime::UNIX_EPOCH.elapsed()?.as_millis()
            ),
        );
    } else {
        check_cmd
            .arg("rustc")
            .args(&ctx.cargo_args)
            .arg("--")
            .arg("--emit")
            .arg("dep-info,metadata")
            // set an always-changing cfg so we can consistently trigger recompile
            .arg("--cfg")
            .arg(format!(
                "__sqlx_recompile_trigger=\"{}\"",
                SystemTime::UNIX_EPOCH.elapsed()?.as_millis()
            ));
    }

    // override database url
    if let Some(database_url) = &ctx.database_url {
        check_cmd.env("DATABASE_URL", database_url);
    }

    check_cmd
        .env("SQLX_OFFLINE", "false")
        .env("SQLX_OFFLINE_DIR", cache_dir);

    println!("executing {:?}", check_cmd);

    let check_status = check_cmd.status()?;

    if !check_status.success() {
        bail!("`cargo check` failed with status: {}", check_status);
    }

    Ok(())
}

fn clear_cache_dir(path: &Path) -> anyhow::Result<()> {
    for entry in fs::read_dir(path)? {
        fs::remove_file(entry?.path())?;
    }

    Ok(())
}
