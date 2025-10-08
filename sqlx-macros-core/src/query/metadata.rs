use sqlx_core::config::Config;
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::query::cache::{MtimeCache, MtimeCacheBuilder};
use sqlx_core::HashMap;

pub struct Metadata {
    pub manifest_dir: PathBuf,
    pub config: Config,
    env: MtimeCache<Arc<MacrosEnv>>,
    workspace_root: Arc<Mutex<Option<PathBuf>>>,
}

pub struct MacrosEnv {
    pub database_url: Option<String>,
    pub offline_dir: Option<PathBuf>,
    pub offline: Option<bool>,
}

impl Metadata {
    pub fn env(&self) -> crate::Result<Arc<MacrosEnv>> {
        self.env
            .get_or_try_init(|builder| load_env(&self.manifest_dir, &self.config, builder))
    }

    pub fn workspace_root(&self) -> PathBuf {
        let mut root = self.workspace_root.lock().unwrap();
        if root.is_none() {
            use serde::Deserialize;
            use std::process::Command;

            let cargo = crate::env("CARGO").unwrap();

            let output = Command::new(cargo)
                .args(["metadata", "--format-version=1", "--no-deps"])
                .current_dir(&self.manifest_dir)
                .env_remove("__CARGO_FIX_PLZ")
                .output()
                .expect("Could not fetch metadata");

            #[derive(Deserialize)]
            struct CargoMetadata {
                workspace_root: PathBuf,
            }

            let metadata: CargoMetadata =
                serde_json::from_slice(&output.stdout).expect("Invalid `cargo metadata` output");

            *root = Some(metadata.workspace_root);
        }
        root.clone().unwrap()
    }
}

pub fn try_for_crate() -> crate::Result<Arc<Metadata>> {
    /// The `MtimeCache` in this type covers the config itself,
    /// any changes to which will indirectly invalidate the loaded env vars as well.
    #[expect(clippy::type_complexity)]
    static METADATA: Mutex<
        HashMap<String, Arc<MtimeCache<Arc<Metadata>>>, BuildHasherDefault<DefaultHasher>>,
    > = Mutex::new(HashMap::with_hasher(BuildHasherDefault::new()));

    let manifest_dir = crate::env("CARGO_MANIFEST_DIR")?;

    let cache = METADATA
        .lock()
        .expect("BUG: we shouldn't panic while holding this lock")
        .entry_ref(&manifest_dir)
        .or_insert_with(|| Arc::new(MtimeCache::new()))
        .clone();

    cache.get_or_try_init(|builder| {
        let manifest_dir = PathBuf::from(manifest_dir);
        let config_path = manifest_dir.join("sqlx.toml");

        builder.add_path(config_path.clone());

        let config = Config::try_from_path_or_default(config_path)?;

        Ok(Arc::new(Metadata {
            manifest_dir,
            config,
            env: MtimeCache::new(),
            workspace_root: Default::default(),
        }))
    })
}

fn load_env(
    manifest_dir: &Path,
    config: &Config,
    builder: &mut MtimeCacheBuilder,
) -> crate::Result<Arc<MacrosEnv>> {
    #[derive(thiserror::Error, Debug)]
    #[error("error reading dotenv file {path:?}")]
    struct DotenvError {
        path: PathBuf,
        #[source]
        error: dotenvy::Error,
    }

    let mut from_dotenv = MacrosEnv {
        database_url: None,
        offline_dir: None,
        offline: None,
    };

    for dir in manifest_dir.ancestors() {
        let path = dir.join(".env");

        let dotenv = match dotenvy::from_path_iter(&path) {
            Ok(iter) => {
                builder.add_path(path.clone());
                iter
            }
            Err(dotenvy::Error::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
                builder.add_path(dir.to_path_buf());
                continue;
            }
            Err(e) => {
                builder.add_path(path.clone());
                return Err(DotenvError { path, error: e }.into());
            }
        };

        for res in dotenv {
            let (name, val) = res.map_err(|e| DotenvError {
                path: path.clone(),
                error: e,
            })?;

            match &*name {
                "SQLX_OFFLINE_DIR" => from_dotenv.offline_dir = Some(val.into()),
                "SQLX_OFFLINE" => from_dotenv.offline = Some(is_truthy_bool(&val)),
                _ if name == config.common.database_url_var() => {
                    from_dotenv.database_url = Some(val)
                }
                _ => continue,
            }
        }
    }

    Ok(Arc::new(MacrosEnv {
        // Make set variables take precedent
        database_url: crate::env_opt(config.common.database_url_var())?
            .or(from_dotenv.database_url),
        offline_dir: crate::env_opt("SQLX_OFFLINE_DIR")?
            .map(PathBuf::from)
            .or(from_dotenv.offline_dir),
        offline: crate::env_opt("SQLX_OFFLINE")?
            .map(|val| is_truthy_bool(&val))
            .or(from_dotenv.offline),
    }))
}

/// Returns `true` if `val` is `"true"`,
fn is_truthy_bool(val: &str) -> bool {
    val.eq_ignore_ascii_case("true") || val == "1"
}
