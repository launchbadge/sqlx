use sqlx_core::connection::{Connect, Connection};
use sqlx_core::database::Database;
use sqlx_core::describe::Describe;
use sqlx_core::executor::{Executor, RefExecutor};
use url::Url;

use std::fmt::{self, Display, Formatter};

use crate::database::DatabaseExt;
use proc_macro2::TokenStream;
use std::fs::File;
use syn::export::Span;

// TODO: enable serialization
#[cfg_attr(feature = "offline", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(
        serialize = "Describe<DB>: serde::Serialize",
        deserialize = "Describe<DB>: serde::de::DeserializeOwned"
    ))
)]
pub struct QueryData<DB: Database> {
    pub(super) query: String,
    pub(super) describe: Describe<DB>,
}

impl<DB: Database> QueryData<DB> {
    pub async fn from_db(
        conn: &mut impl Executor<Database = DB>,
        query: &str,
    ) -> crate::Result<Self> {
        Ok(QueryData {
            query: query.into(),
            describe: conn.describe(query).await?,
        })
    }
}

#[cfg(feature = "offline")]
pub mod offline {
    use super::QueryData;
    use std::fs::File;

    use std::fmt::{self, Formatter};

    use crate::database::DatabaseExt;
    use proc_macro2::{Span, TokenStream};
    use serde::de::{Deserializer, MapAccess, Visitor};
    use sqlx_core::describe::Describe;
    use sqlx_core::query::query;
    use std::path::Path;

    #[derive(serde::Deserialize)]
    pub struct DynQueryData {
        #[serde(skip)]
        pub db_name: String,
        pub query: String,
        pub describe: serde_json::Value,
    }

    impl DynQueryData {
        /// Find and deserialize the data table for this query from a shared `sqlx-data.json`
        /// file. The expected structure is a JSON map keyed by the SHA-256 hash of queries in hex.
        pub fn from_data_file(path: impl AsRef<Path>, query: &str) -> crate::Result<Self> {
            serde_json::Deserializer::from_reader(
                File::open(path.as_ref()).map_err(|e| {
                    format!("failed to open path {}: {}", path.as_ref().display(), e)
                })?,
            )
            .deserialize_map(DataFileVisitor {
                query,
                hash: hash_string(query),
            })
            .map_err(Into::into)
        }
    }

    impl<DB: DatabaseExt> QueryData<DB>
    where
        Describe<DB>: serde::Serialize + serde::de::DeserializeOwned,
    {
        pub fn from_dyn_data(dyn_data: DynQueryData) -> crate::Result<Self> {
            assert!(!dyn_data.db_name.is_empty());
            if DB::NAME == dyn_data.db_name {
                let describe: Describe<DB> = serde_json::from_value(dyn_data.describe)?;
                Ok(QueryData {
                    query: dyn_data.query,
                    describe,
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
                File::create(&path)
                    .map_err(|e| format!("failed to open path {}: {}", path.display(), e))?,
                self,
            )
            .map_err(Into::into)
        }
    }

    fn hash_string(query: &str) -> String {
        // picked `sha2` because it's already in the dependency tree for both MySQL and Postgres
        use sha2::{Digest, Sha256};

        hex::encode(Sha256::digest(query.as_bytes()))
    }

    // lazily deserializes only the `QueryData` for the query we're looking for
    struct DataFileVisitor<'a> {
        query: &'a str,
        hash: String,
    }

    impl<'de> Visitor<'de> for DataFileVisitor<'_> {
        type Value = DynQueryData;

        fn expecting(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "expected map key {:?} or \"db\"", self.hash)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
        where
            A: MapAccess<'de>,
        {
            let mut db_name: Option<String> = None;

            // unfortunately we can't avoid this copy because deserializing from `io::Read`
            // doesn't support deserializing borrowed values
            while let Some(key) = map.next_key::<String>()? {
                // lazily deserialize the query data only
                if key == "db" {
                    db_name = Some(map.next_value::<String>()?);
                } else if key == self.hash {
                    let db_name = db_name.ok_or_else(|| {
                        serde::de::Error::custom("expected \"db\" key before query hash keys")
                    })?;

                    let mut query_data: DynQueryData = map.next_value()?;

                    return if query_data.query == self.query {
                        query_data.db_name = db_name;
                        Ok(query_data)
                    } else {
                        Err(serde::de::Error::custom(format_args!(
                            "hash collision for stored queries:\n{:?}\n{:?}",
                            self.query, query_data.query
                        )))
                    };
                }
            }

            Err(serde::de::Error::custom(format_args!(
                "failed to find data for query {}",
                self.hash
            )))
        }
    }
}
