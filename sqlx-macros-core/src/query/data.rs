use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::Write as _;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::{Serialize, Serializer};

use sqlx_core::database::Database;
use sqlx_core::describe::Describe;

use crate::database::DatabaseExt;

#[derive(serde::Serialize)]
#[serde(bound(serialize = "Describe<DB>: serde::Serialize"))]
#[derive(Debug)]
pub struct QueryData<DB: Database> {
    db_name: SerializeDbName<DB>,
    #[allow(dead_code)]
    pub(super) query: String,
    pub(super) describe: Describe<DB>,
    pub(super) hash: String,
}

impl<DB: Database> QueryData<DB> {
    pub fn from_describe(query: &str, describe: Describe<DB>) -> Self {
        QueryData {
            db_name: SerializeDbName::default(),
            query: query.into(),
            describe,
            hash: hash_string(query),
        }
    }
}

struct SerializeDbName<DB>(PhantomData<DB>);

impl<DB> Default for SerializeDbName<DB> {
    fn default() -> Self {
        SerializeDbName(PhantomData)
    }
}

impl<DB: Database> Debug for SerializeDbName<DB> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SerializeDbName").field(&DB::NAME).finish()
    }
}

impl<DB: Database> Display for SerializeDbName<DB> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(DB::NAME)
    }
}

impl<DB: Database> Serialize for SerializeDbName<DB> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(DB::NAME)
    }
}

static OFFLINE_DATA_CACHE: Lazy<Mutex<HashMap<PathBuf, DynQueryData>>> =
    Lazy::new(Default::default);

/// Offline query data
#[derive(Clone, serde::Deserialize)]
pub struct DynQueryData {
    pub db_name: String,
    pub query: String,
    pub describe: serde_json::Value,
    pub hash: String,
}

impl DynQueryData {
    /// Loads a query given the path to its "query-<hash>.json" file. Subsequent calls for the same
    /// path are retrieved from an in-memory cache.
    pub fn from_data_file(path: impl AsRef<Path>, query: &str) -> crate::Result<Self> {
        let path = path.as_ref();

        let mut cache = OFFLINE_DATA_CACHE
            .lock()
            // Just reset the cache on error
            .unwrap_or_else(|posion_err| {
                let mut guard = posion_err.into_inner();
                *guard = Default::default();
                guard
            });
        if let Some(cached) = cache.get(path).cloned() {
            if query != cached.query {
                return Err("hash collision for saved query data".into());
            }
            return Ok(cached);
        }

        #[cfg(procmacr2_semver_exempt)]
        {
            let path = path.as_ref().canonicalize()?;
            let path = path.to_str().ok_or_else(|| {
                format!(
                    "query-<hash>.json path cannot be represented as a string: {:?}",
                    path
                )
            })?;

            proc_macro::tracked_path::path(path);
        }

        let offline_data_contents = fs::read_to_string(path)
            .map_err(|e| format!("failed to read saved query path {}: {}", path.display(), e))?;
        let dyn_data: DynQueryData = serde_json::from_str(&offline_data_contents)?;

        if query != dyn_data.query {
            return Err("hash collision for saved query data".into());
        }

        let _ = cache.insert(path.to_owned(), dyn_data.clone());
        Ok(dyn_data)
    }
}

impl<DB: DatabaseExt> QueryData<DB>
where
    Describe<DB>: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn from_dyn_data(dyn_data: DynQueryData) -> crate::Result<Self> {
        assert!(!dyn_data.db_name.is_empty());
        assert!(!dyn_data.hash.is_empty());

        if DB::NAME == dyn_data.db_name {
            let describe: Describe<DB> = serde_json::from_value(dyn_data.describe)?;
            Ok(QueryData {
                db_name: SerializeDbName::default(),
                query: dyn_data.query,
                describe,
                hash: dyn_data.hash,
            })
        } else {
            Err(format!(
                "expected query data for {}, got data for {}",
                DB::NAME,
                dyn_data.db_name
            )
            .into())
        }
    }

    pub(super) fn save_in(&self, dir: impl AsRef<Path>) -> crate::Result<()> {
        fn inner<DB>(data: &QueryData<DB>, file: &mut std::fs::File) -> crate::Result<()>
        where
            DB: DatabaseExt,
            Describe<DB>: serde::Serialize + serde::de::DeserializeOwned,
        {
            serde_json::to_writer_pretty(&mut *file, data)
                .map_err(|err| format!("failed to serialize query data to file: {err:?}"))?;

            // Ensure there is a newline at the end of the JSON file to avoid accidental modification by IDE
            // and make github diff tool happier.
            file.write_all(b"\n")
                .map_err(|err| format!("failed to append a newline to file: {err:?}"))?;

            Ok(())
        }

        let path = dir.as_ref().join(format!("query-{}.json", self.hash));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|err| format!("failed to create query file: {err:?}"))?;

        let res = inner(self, &mut file);

        if res.is_err() {
            _ = std::fs::remove_file(&path);
        }

        res
    }
}

pub(super) fn hash_string(query: &str) -> String {
    // picked `sha2` because it's already in the dependency tree for both MySQL and Postgres
    use sha2::{Digest, Sha256};

    hex::encode(Sha256::digest(query.as_bytes()))
}
