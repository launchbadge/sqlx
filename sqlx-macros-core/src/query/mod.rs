use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, io};

use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use syn::Type;

pub use input::QueryMacroInput;
use quote::{format_ident, quote};
use sqlx_core::database::Database;
use sqlx_core::{column::Column, describe::Describe, type_info::TypeInfo};

use crate::database::DatabaseExt;
use crate::query::data::{hash_string, DynQueryData, QueryData};
use crate::query::input::RecordType;
use either::Either;
use url::Url;

mod args;
mod data;
mod input;
mod output;

#[derive(Copy, Clone)]
pub struct QueryDriver {
    db_name: &'static str,
    url_schemes: &'static [&'static str],
    expand: fn(QueryMacroInput, QueryDataSource) -> crate::Result<TokenStream>,
}

impl QueryDriver {
    pub const fn new<DB: DatabaseExt>() -> Self
    where
        Describe<DB>: serde::Serialize + serde::de::DeserializeOwned,
    {
        QueryDriver {
            db_name: DB::NAME,
            url_schemes: DB::URL_SCHEMES,
            expand: expand_with::<DB>,
        }
    }
}
pub enum QueryDataSource<'a> {
    Live {
        database_url: &'a str,
        database_url_parsed: Url,
    },
    Cached(DynQueryData),
}

impl<'a> QueryDataSource<'a> {
    pub fn live(database_url: &'a str) -> crate::Result<Self> {
        Ok(QueryDataSource::Live {
            database_url,
            database_url_parsed: database_url.parse()?,
        })
    }

    pub fn matches_driver(&self, driver: &QueryDriver) -> bool {
        match self {
            Self::Live {
                database_url_parsed,
                ..
            } => driver.url_schemes.contains(&database_url_parsed.scheme()),
            Self::Cached(dyn_data) => dyn_data.db_name == driver.db_name,
        }
    }
}

struct Metadata {
    #[allow(unused)]
    manifest_dir: PathBuf,
    offline: bool,
    default_database_url: Option<String>,
    workspace_root: Arc<Mutex<Option<PathBuf>>>,
    env_cache: HashMap<String, String>,
}

impl Metadata {
    pub fn workspace_root(&self) -> PathBuf {
        let mut root = self.workspace_root.lock().unwrap();
        if root.is_none() {
            use serde::Deserialize;
            use std::process::Command;

            let cargo = env("CARGO").expect("`CARGO` must be set");

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

// If we are in a workspace, lookup `workspace_root` since `CARGO_MANIFEST_DIR` won't
// reflect the workspace dir: https://github.com/rust-lang/cargo/issues/3946
static METADATA: Lazy<Metadata> = Lazy::new(|| {
    let manifest_dir: PathBuf = env("CARGO_MANIFEST_DIR")
        .expect("`CARGO_MANIFEST_DIR` must be set")
        .into();

    // If a .env file exists at CARGO_MANIFEST_DIR, load environment variables from this,
    // otherwise fallback to default dotenv behaviour.
    let env_path = manifest_dir.join(".env");

    #[cfg_attr(not(procmacro2_semver_exempt), allow(unused_variables))]
    let env_path = if env_path.exists() {
        let res = dotenvy::from_path(&env_path);
        if let Err(e) = res {
            panic!("failed to load environment from {env_path:?}, {e}");
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

    let env_cache = HashMap::from_iter(dotenvy::vars());

    let default_database_url = env("DATABASE_URL").ok();

    Metadata {
        manifest_dir,
        offline,
        default_database_url,
        workspace_root: Arc::new(Mutex::new(None)),
        env_cache,
    }
});

pub fn expand_input<'a>(
    input: QueryMacroInput,
    drivers: impl IntoIterator<Item = &'a QueryDriver>,
) -> crate::Result<TokenStream> {
    // If we don't require the query to be offline, check if we have a valid online datasource url
    let online_data_source: Option<QueryDataSource> = if METADATA.offline == false {
        if let Some(ref custom_env) = input.db_url_env {
            // Get the custom db url environment
            METADATA
                .env_cache
                .get(custom_env)
                .map(|custom_db_url| QueryDataSource::live(custom_db_url))
                .transpose()?
        } else if let Some(default_database_url) = &METADATA.default_database_url {
            // Get the default db url env
            Some(QueryDataSource::live(default_database_url)?)
        } else {
            None
        }
    } else {
        None
    };

    let data_source = if let Some(data_source) = online_data_source {
        data_source
    } else {
        // If we don't have a live source, try load the cached query metadata file.
        let filename = format!("query-{}.json", hash_string(&input.sql));

        // Check SQLX_OFFLINE_DIR, then local .sqlx, then workspace .sqlx.
        let dirs = [
            || env("SQLX_OFFLINE_DIR").ok().map(PathBuf::from),
            || Some(METADATA.manifest_dir.join(".sqlx")),
            || Some(METADATA.workspace_root().join(".sqlx")),
        ];
        let Some(data_file_path) = dirs
            .iter()
            .filter_map(|path| path())
            .map(|path| {
                if let Some(ref custom_env) = input.db_url_env {
                    path.join(custom_env).join(&filename)
                } else {
                    path.join(&filename)
                }
            })
            .find(|path| path.exists())
        else {
            return Err(
                if METADATA.offline {
                    "`SQLX_OFFLINE=true` but there is no cached data for this query, run `cargo sqlx prepare` to update the query cache or unset `SQLX_OFFLINE`".to_string()
                } else {
                    if let Some(custom_env) = input.db_url_env {
                        format!("set custom env `{:?}` to use query macros online, or run `cargo sqlx prepare` to update the query cache", custom_env)
                    } else {
                        "set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` to update the query cache".to_string()
                    }
                }.into()
            );
        };

        QueryDataSource::Cached(DynQueryData::from_data_file(&data_file_path, &input.sql)?)
    };

    for driver in drivers {
        if data_source.matches_driver(driver) {
            return (driver.expand)(input, data_source);
        }
    }

    match data_source {
        QueryDataSource::Live {
            database_url_parsed,
            ..
        } => Err(format!(
            "no database driver found matching URL scheme {:?}; the corresponding Cargo feature may need to be enabled", 
            database_url_parsed.scheme()
        ).into()),
        QueryDataSource::Cached(data) => {
            Err(format!(
                "found cached data for database {:?} but no matching driver; the corresponding Cargo feature may need to be enabled",
                data.db_name
            ).into())
        }
    }
}

fn expand_with<DB: DatabaseExt>(
    input: QueryMacroInput,
    data_source: QueryDataSource,
) -> crate::Result<TokenStream>
where
    Describe<DB>: DescribeExt,
{
    let (query_data, offline): (QueryData<DB>, bool) = match data_source {
        QueryDataSource::Cached(dyn_data) => (QueryData::from_dyn_data(dyn_data)?, true),
        QueryDataSource::Live { database_url, .. } => {
            let describe = DB::describe_blocking(&input.sql, database_url)?;
            (QueryData::from_describe(&input.sql, describe), false)
        }
    };

    expand_with_data(input, query_data, offline)
}

// marker trait for `Describe` that lets us conditionally require it to be `Serialize + Deserialize`
trait DescribeExt: serde::Serialize + serde::de::DeserializeOwned {}

impl<DB: Database> DescribeExt for Describe<DB> where
    Describe<DB>: serde::Serialize + serde::de::DeserializeOwned
{
}

fn expand_with_data<DB: DatabaseExt>(
    input: QueryMacroInput,
    data: QueryData<DB>,
    offline: bool,
) -> crate::Result<TokenStream>
where
    Describe<DB>: DescribeExt,
{
    // validate at the minimum that our args match the query's input parameters
    let num_parameters = match data.describe.parameters() {
        Some(Either::Left(params)) => Some(params.len()),
        Some(Either::Right(num)) => Some(num),

        None => None,
    };

    if let Some(num) = num_parameters {
        if num != input.arg_exprs.len() {
            return Err(
                format!("expected {} parameters, got {}", num, input.arg_exprs.len()).into(),
            );
        }
    }

    let args_tokens = args::quote_args(&input, &data.describe)?;

    let query_args = format_ident!("query_args");

    let output = if data
        .describe
        .columns()
        .iter()
        .all(|it| it.type_info().is_void())
    {
        let db_path = DB::db_path();
        let sql = &input.sql;

        quote! {
            ::sqlx::__query_with_result::<#db_path, _>(#sql, #query_args)
        }
    } else {
        match input.record_type {
            RecordType::Generated => {
                let columns = output::columns_to_rust::<DB>(&data.describe)?;

                let record_name: Type = syn::parse_str("Record").unwrap();

                for rust_col in &columns {
                    if rust_col.type_.is_wildcard() {
                        return Err(
                            "wildcard overrides are only allowed with an explicit record type, \
                             e.g. `query_as!()` and its variants"
                                .into(),
                        );
                    }
                }

                let record_fields = columns
                    .iter()
                    .map(|output::RustColumn { ident, type_, .. }| quote!(#ident: #type_,));

                let mut record_tokens = quote! {
                    #[derive(Debug)]
                    struct #record_name {
                        #(#record_fields)*
                    }
                };

                record_tokens.extend(output::quote_query_as::<DB>(
                    &input,
                    &record_name,
                    &query_args,
                    &columns,
                ));

                record_tokens
            }
            RecordType::Given(ref out_ty) => {
                let columns = output::columns_to_rust::<DB>(&data.describe)?;

                output::quote_query_as::<DB>(&input, out_ty, &query_args, &columns)
            }
            RecordType::Scalar => {
                output::quote_query_scalar::<DB>(&input, &query_args, &data.describe)?
            }
        }
    };

    let ret_tokens = quote! {
        {
            #[allow(clippy::all)]
            {
                use ::sqlx::Arguments as _;

                #args_tokens

                #output
            }
        }
    };

    // Store query metadata only if offline support is enabled but the current build is online.
    // If the build is offline, the cache is our input so it's pointless to also write data for it.
    if !offline {
        // Only save query metadata if SQLX_OFFLINE_DIR is set manually or by `cargo sqlx prepare`.
        // Note: in a cargo workspace this path is relative to the root.
        if let Ok(dir) = env("SQLX_OFFLINE_DIR") {
            let path = PathBuf::from(&dir);

            match fs::metadata(&path) {
                Err(e) => {
                    if e.kind() != io::ErrorKind::NotFound {
                        // Can't obtain information about .sqlx
                        return Err(format!("{e}: {dir}").into());
                    }
                    // .sqlx doesn't exist.
                    return Err(format!("sqlx offline path does not exist: {dir}").into());
                }
                Ok(meta) => {
                    if !meta.is_dir() {
                        return Err(format!(
                            "sqlx offline path exists, but is not a directory: {dir}"
                        )
                        .into());
                    }

                    if let Some(custom_db_env) = input.db_url_env {
                        let full_path: PathBuf = path.join(custom_db_env);
     
                        match fs::create_dir(&full_path) {
                            Ok(_) => {}
                            Err(err) => {
                                match err.kind() {
                                    std::io::ErrorKind::AlreadyExists => {}
                                    _ => return Err(format!(
                                        "Failed to create offline cache path {full_path:?}: {err}"
                                    )
                                    .into()),
                                }
                            }
                        };

                        // created subfolder if not exists, store data.
                        data.save_in(full_path)?;
                    } else {
                        // .sqlx exists and is a directory, store data.
                        data.save_in(path)?;
                    }
                }
            }
        }
    }

    Ok(ret_tokens)
}

/// Get the value of an environment variable, telling the compiler about it if applicable.
fn env(name: &str) -> Result<String, std::env::VarError> {
    #[cfg(procmacro2_semver_exempt)]
    {
        proc_macro::tracked_env::var(name)
    }

    #[cfg(not(procmacro2_semver_exempt))]
    {
        std::env::var(name)
    }
}
