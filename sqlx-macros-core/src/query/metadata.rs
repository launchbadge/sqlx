use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use once_cell::sync::{Lazy, OnceCell};
use crate::query::{config, env};
use crate::query::config::Config;

pub struct Metadata {
    #[allow(unused)]
    pub manifest_dir: PathBuf,
    pub offline: bool,
    pub config: Config,
    pub database_url: Option<String>,
    workspace_root: OnceCell<PathBuf>,
}

impl Metadata {
    pub fn get() -> crate::Result<&'static Self> {
        static METADATA: OnceCell<Metadata> = OnceCell::new();
        METADATA.get_or_try_init(Self::init)
    }

    pub fn url_var(&self) -> &str {
        self.config.url_var.as_deref().unwrap_or("DATABASE_URL")
    }

    // If we are in a workspace, lookup `workspace_root` since `CARGO_MANIFEST_DIR` won't
    // reflect the workspace dir: https://github.com/rust-lang/cargo/issues/3946
    pub fn workspace_root(&self) -> crate::Result<&PathBuf> {
        self.workspace_root.get_or_try_init(|| {
            use serde::Deserialize;
            use std::process::Command;

            let cargo = env("CARGO").map_err(|_| "`CARGO` must be set")?;

            let output = Command::new(&cargo)
                .args(&["metadata", "--format-version=1", "--no-deps"])
                .current_dir(&self.manifest_dir)
                .env_remove("__CARGO_FIX_PLZ")
                .output()
                .map_err(|e| format!("Could not fetch metadata: {e:?}"))?;

            #[derive(Deserialize)]
            struct CargoMetadata {
                workspace_root: PathBuf,
            }

            let metadata: CargoMetadata =
                serde_json::from_slice(&output.stdout)
                    .map_err(|e| format!("Invalid `cargo metadata` output: {e:?}"))?;

            Ok(metadata.workspace_root)
        })
    }

    fn init() -> crate::Result<Self> {
        let manifest_dir: PathBuf = env("CARGO_MANIFEST_DIR")
            .map_err(|| "`CARGO_MANIFEST_DIR` must be set")?
            .into();

        let config_path = manifest_dir.join("sqlx.toml");

        let config = if config_path.exists() {
            config::load(&config_path)
                .map_err(|e| format!("failed to load config at {}: {e:?}", config_path.display()))?
        } else {
            Config::default()
        };

        // If a .env file exists at CARGO_MANIFEST_DIR, load environment variables from this,
        // otherwise fallback to default dotenv behaviour.
        let env_path = manifest_dir.join(".env");

        #[cfg_attr(not(procmacro2_semver_exempt), allow(unused_variables))]
        let env_path = if env_path.exists() {
            let res = dotenvy::from_path(&env_path);
            if let Err(e) = res {
                return Err(format!("failed to load environment from {env_path:?}, {e}").into());
            }

            Some(env_path)
        } else {
            dotenvy::dotenv().ok()
        };

        // tell the compiler to watch the `.env` for changes, if applicable
        #[cfg(procmacro2_semver_exempt)]
        if let Some(env_path) = env_path.as_ref().and_then(|path| path.to_str()) {
            proc_macro::tracked_path::path(env_path);
        }

        let offline = env("SQLX_OFFLINE")
            .map(|s| s.eq_ignore_ascii_case("true") || s == "1")
            .unwrap_or(false);

        let database_url = env(config.url_var.as_deref().unwrap_or("DATABASE_URL")).ok();

        Ok(Metadata {
            manifest_dir,
            offline,
            config,
            database_url,
            workspace_root: Arc::new(Mutex::new(None)),
        })
    }
}
