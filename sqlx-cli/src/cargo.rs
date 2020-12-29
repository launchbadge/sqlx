use anyhow::Context;
use serde::Deserialize;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;
use std::str;

#[derive(Deserialize)]
pub struct CargoMetadata {
    pub target_directory: PathBuf,
    pub workspace_root: PathBuf,
}

/// Path to the `cargo` executable
pub fn cargo_path() -> anyhow::Result<OsString> {
    env::var_os("CARGO").context("Failed to obtain value of `CARGO`")
}

pub fn manifest_dir(cargo: &OsStr) -> anyhow::Result<PathBuf> {
    let stdout = Command::new(&cargo)
        .args(&["locate-project", "--message-format=plain"])
        .output()
        .context("could not locate manifest dir")?
        .stdout;

    let mut manifest_path: PathBuf = str::from_utf8(&stdout)
        .context("output of `cargo locate-project` was not valid UTF-8")?
        // get rid of the trailing newline
        .trim()
        .into();

    manifest_path.pop();

    Ok(manifest_path)
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
