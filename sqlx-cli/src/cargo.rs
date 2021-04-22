use anyhow::Context;
use serde::Deserialize;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;

#[derive(Deserialize)]
pub struct CargoMetadata {
    pub target_directory: PathBuf,
    pub workspace_root: PathBuf,
}

/// Path to the `cargo` executable
pub fn cargo_path() -> anyhow::Result<OsString> {
    env::var_os("CARGO").context("Failed to obtain value of `CARGO`")
}

pub fn manifest_dir() -> anyhow::Result<PathBuf> {
    Ok(env::var_os("CARGO_MANIFEST_DIR")
        .context("Failed to obtain value of `CARGO_MANIFEST_DIR`")?
        .into())
}

pub fn metadata(cargo: &OsStr) -> anyhow::Result<CargoMetadata> {
    let output = Command::new(&cargo)
        .args(&["metadata", "--format-version=1"])
        .output()
        .context("Could not fetch metadata")?;

    serde_json::from_slice(&output.stdout)
        .context("Invalid `cargo metadata` output")
        .map_err(Into::into)
}
