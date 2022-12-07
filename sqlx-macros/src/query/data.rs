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
        Ok(Self::from_describe(query, conn.describe(query).await?))
    }

    pub fn from_describe(query: &str, describe: Describe<DB>) -> Self {
        QueryData {
            query: query.into(),
            describe,
            #[cfg(feature = "offline")]
            hash: offline::hash_string(query),
        }
    }
}

#[cfg(feature = "offline")]
pub mod offline {
    use super::QueryData;
    use crate::database::DatabaseExt;

    use std::collections::BTreeMap;
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use proc_macro2::Span;
    use sqlx_core::describe::Describe;

    static OFFLINE_DATA_CACHE: Lazy<Mutex<BTreeMap<PathBuf, OfflineData>>> =
        Lazy::new(|| Mutex::new(BTreeMap::new()));

    #[derive(serde::Deserialize)]
    struct BaseQuery {
        query: String,
        describe: serde_json::Value,
    }

    #[derive(serde::Deserialize)]
    struct OfflineData {
        db: String,
        #[serde(flatten)]
        hash_to_query: BTreeMap<String, BaseQuery>,
    }

    impl OfflineData {
        fn get_query_from_hash(&self, hash: &str) -> Option<DynQueryData> {
            self.hash_to_query.get(hash).map(|base_query| DynQueryData {
                db_name: self.db.clone(),
                query: base_query.query.to_owned(),
                describe: base_query.describe.to_owned(),
                hash: hash.to_owned(),
            })
        }
    }

    #[derive(serde::Deserialize)]
    pub struct DynQueryData {
        #[serde(skip)]
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
            let path = path.as_ref();

            let query_data = {
                let mut cache = OFFLINE_DATA_CACHE
                    .lock()
                    // Just reset the cache on error
                    .unwrap_or_else(|posion_err| {
                        let mut guard = posion_err.into_inner();
                        *guard = BTreeMap::new();
                        guard
                    });

                if !cache.contains_key(path) {
                    let offline_data_contents = fs::read_to_string(path)
                        .map_err(|e| format!("failed to read path {}: {}", path.display(), e))?;
                    let offline_data: OfflineData = serde_json::from_str(&offline_data_contents)?;
                    let _ = cache.insert(path.to_owned(), offline_data);
                }

                let offline_data = cache
                    .get(path)
                    .expect("Missing data should have just been added");

                let query_hash = hash_string(query);
                let query_data = offline_data
                    .get_query_from_hash(&query_hash)
                    .ok_or_else(|| format!("failed to find data for query {}", query))?;

                if query != query_data.query {
                    return Err(format!(
                        "hash collision for stored queries:\n{:?}\n{:?}",
                        query, query_data.query
                    )
                    .into());
                }

                query_data
            };

            #[cfg(procmacr2_semver_exempt)]
            {
                let path = path.as_ref().canonicalize()?;
                let path = path.to_str().ok_or_else(|| {
                    format!(
                        "sqlx-data.json path cannot be represented as a string: {:?}",
                        path
                    )
                })?;

                proc_macro::tracked_path::path(path);
            }

            Ok(query_data)
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

        pub fn save_in(&self, dir: impl AsRef<Path>, input_span: Span) -> crate::Result<()> {
            // we save under the hash of the span representation because that should be unique
            // per invocation
            let path = dir.as_ref().join(format!(
                "query-{}.json",
                hash_string(&format!("{:?}", input_span))
            ));

            serde_json::to_writer_pretty(
                BufWriter::new(
                    File::create(&path)
                        .map_err(|e| format!("failed to open path {}: {}", path.display(), e))?,
                ),
                self,
            )
            .map_err(Into::into)
        }
    }

    pub fn hash_string(query: &str) -> String {
        // picked `sha2` because it's already in the dependency tree for both MySQL and Postgres
        use sha2::{Digest, Sha256};

        hex::encode(Sha256::digest(query.as_bytes()))
    }
}
