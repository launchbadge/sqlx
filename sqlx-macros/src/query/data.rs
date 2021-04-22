use sqlx_core::database::Database;
use sqlx_core::describe::Describe;
use sqlx_core::executor::Executor;

#[cfg_attr(feature = "offline", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(
        serialize = "Describe<DB>: serde::Serialize",
        deserialize = "Describe<DB>: serde::de::DeserializeOwned"
    ))
)]
#[derive(Debug)]
pub struct QueryData<DB: Database> {
    #[allow(dead_code)]
    pub(super) query: String,
    pub(super) describe: Describe<DB>,
    #[cfg(feature = "offline")]
    pub(super) hash: String,
}

impl<DB: Database> QueryData<DB> {
    pub async fn from_db(
        conn: impl Executor<'_, Database = DB>,
        query: &str,
    ) -> crate::Result<Self> {
        Ok(QueryData {
            query: query.into(),
            describe: conn.describe(query).await?,
            #[cfg(feature = "offline")]
            hash: super::hash_string(query),
        })
    }
}

#[cfg(feature = "offline")]
pub mod offline {
    use super::QueryData;
    use crate::database::DatabaseExt;
    use crate::query::hash_string;

    use std::fs;
    use std::io::BufWriter;
    use std::path::Path;

    use sqlx_core::describe::Describe;
    use tempfile::NamedTempFile;

    #[derive(serde::Deserialize)]
    pub struct DynQueryData {
        pub db_name: String,
        pub query: String,
        pub describe: serde_json::Value,
        #[serde(skip)]
        pub hash: String,
    }

    impl DynQueryData {
        /// Find and deserialize the data table for this query from a shared `sqlx-data.json`
        /// file. The expected structure is a JSON map keyed by the SHA-256 hash of queries in hex.
        pub fn from_data_file(path: impl AsRef<Path>, query: &str) -> crate::Result<Self> {
            // It's faster to read the whole file into memory first instead of deserializing from an
            // `io::Read` instance using `from_reader`: https://github.com/serde-rs/json/issues/160
            let json_s = fs::read_to_string(path.as_ref())
                .map_err(|e| format!("failed to open path {}: {}", path.as_ref().display(), e))?;

            Ok(Self {
                hash: hash_string(query),
                ..serde_json::from_str(&json_s)?
            })
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

        pub fn save_in(&self, dir: impl AsRef<Path>) -> crate::Result<()> {
            let dir = dir.as_ref();

            // We first write to a temporary file to then move it to the final location.
            // This ensures no file corruption happens in case this method is called concurrently
            // for the same query.
            let file = NamedTempFile::new_in(dir).map_err(|e| {
                format!(
                    "failed to create temporary file in {}: {}",
                    dir.display(),
                    e,
                )
            })?;

            serde_json::to_writer_pretty(BufWriter::new(&file), self)?;

            let path = dir.join(format!("query-{}.json", hash_string(&self.query)));
            file.persist(&path).map_err(|e| {
                format!(
                    "failed to move temporary file {} to {}: {}",
                    e.file.path().display(),
                    path.display(),
                    e.error,
                )
            })?;

            Ok(())
        }
    }
}
