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
    db_type_name: &'static str,
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
            db_type_name: DB::TYPE_IMPORT_PATH,
        }
    }
}

#[derive(Clone)]
pub struct QueryDataSourceUrl<'a> {
    database_url: &'a str,
    database_url_parsed: Url,
}

impl<'a> From<&'a String> for QueryDataSourceUrl<'a> {
    fn from(database_url: &'a String) -> Self {
        let database_url_parsed = Url::parse(database_url).expect("invalid URL");

        QueryDataSourceUrl {
            database_url,
            database_url_parsed,
        }
    }
}

#[derive(Clone)]
pub enum QueryDataSource<'a> {
    Live {
        database_urls: Vec<QueryDataSourceUrl<'a>>,  
    },
    Cached(DynQueryData),
}

impl<'a> QueryDataSource<'a> {
    pub fn live(database_urls: Vec<QueryDataSourceUrl<'a>>) -> crate::Result<Self> {
        Ok(QueryDataSource::Live {
            database_urls,
        })
    }

    pub fn matches_driver(&self, driver: &QueryDriver) -> bool {
        match self {
            Self::Live {
                database_urls,
                ..
            } => driver.url_schemes.iter().any(|scheme| {
                database_urls.iter().any(|url| url.database_url_parsed.scheme() == *scheme)
            }),
            Self::Cached(dyn_data) => dyn_data.db_name == driver.db_name,
        }
    }

    pub fn get_url_for_schemes(&self, schemes: &[&str]) -> Option<&QueryDataSourceUrl> {
        match self {
            Self::Live {
                database_urls,
                ..
            } => {
                for scheme in schemes {
                    if let Some(url) = database_urls.iter().find(|url| url.database_url_parsed.scheme() == *scheme) {
                        return Some(url);
                    }
                }
                None
            }
            Self::Cached(_) => {
                None
            }
        }
    }

    pub fn supported_schemes(&self) -> Vec<&str> {
        match self {
            Self::Live {
                 database_urls,
                ..
            } => {
                let mut schemes = vec![];
                schemes.extend(database_urls.iter().map(|url| url.database_url_parsed.scheme()));
                schemes
            }
            Self::Cached(..) => vec![],
        }
    }
}

struct Metadata {
    #[allow(unused)]
    manifest_dir: PathBuf,
    offline: bool,
    database_urls: Vec<String>,
    workspace_root: Arc<Mutex<Option<PathBuf>>>,
}

impl Metadata {
    pub fn workspace_root(&self) -> PathBuf {
        let mut root = self.workspace_root.lock().unwrap();
        if root.is_none() {
            use serde::Deserialize;
            use std::process::Command;

            let cargo = env("CARGO").expect("`CARGO` must be set");

            let output = Command::new(&cargo)
                .args(&["metadata", "--format-version=1", "--no-deps"])
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

    Metadata {
        manifest_dir,
        offline,
        database_urls: env_db_urls(),
        workspace_root: Arc::new(Mutex::new(None)),
    }
});

pub fn expand_input<'a>(
    input: QueryMacroInput,
    drivers: impl IntoIterator<Item = &'a QueryDriver>,
) -> crate::Result<TokenStream> {
    let data_source = match &*METADATA {
        Metadata {
            offline: false,
            database_urls: db_urls,
            ..
        } => {
            QueryDataSource::live(db_urls.iter().map(QueryDataSourceUrl::from).collect())?
        },

        Metadata { offline, .. } => {
            // Try load the cached query metadata file.
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
                .map(|path| path.join(&filename))
                .find(|path| path.exists())
            else {
                return Err(
                    if *offline {
                        "`SQLX_OFFLINE=true` but there is no cached data for this query, run `cargo sqlx prepare` to update the query cache or unset `SQLX_OFFLINE`"
                    } else {
                        "set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` to update the query cache"
                    }.into()
                );
            };

            QueryDataSource::Cached(DynQueryData::from_data_file(&data_file_path, &input.sql)?)
        }
    };

    let mut working_drivers = vec![];

    // If the driver was explicitly set, use it directly.
    if let Some(input_driver) = input.driver.clone() {
        for driver in drivers {
                if (driver.expand)(input.clone(), data_source.clone()).is_ok() {
                    working_drivers.push(driver);
                }
        }

        return match working_drivers.len() {
            0 => {
                Err(format!(
                    "no database driver found matching for query; the corresponding Cargo feature may need to be enabled"
                ).into())
            }
            1 => {
                let driver = working_drivers.pop().unwrap();
                (driver.expand)(input, data_source)
            }
            _ => {
                let expansions = working_drivers.iter().map(|driver| {
                    let driver_name = driver.db_type_name;
                    let driver_type: Type = syn::parse_str(driver_name).unwrap();
                    let expanded = (driver.expand)(input.clone(), data_source.clone()).unwrap();
                    quote! {
                        impl ProvideQuery<#driver_type> for #driver_type {
                            fn provide_query<'a>() -> Query<'a, #driver_type, <#driver_type as sqlx::Database>::Arguments<'a>> {
                                #expanded
                            }
                        }
                    }
                });
                Ok(quote! {
                    {
                        use sqlx::query::Query;
                        trait ProvideQuery<DB: sqlx::Database> {
                            fn provide_query<'a>() -> Query<'a, DB, DB::Arguments<'a>>;
                        }
                        #(#expansions)*
                        #input_driver::provide_query()
                    }
                })
            }
        }
    }
    
    // If no driver was set, try to find a matching driver for the data source.
    for driver in drivers {
        if data_source.matches_driver(driver) {
            let result = (driver.expand)(input, data_source);
            return result;
        }
    }

    match data_source {
        QueryDataSource::Live{..} => Err(format!(
            "no database driver found matching URL scheme {:?}; the corresponding Cargo feature may need to be enabled", 
            data_source.supported_schemes()
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
        QueryDataSource::Live { .. } => {
            let data_source_url = data_source.get_url_for_schemes(DB::URL_SCHEMES).unwrap();
            let describe = DB::describe_blocking(&input.sql, data_source_url.database_url)?;
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

                let record_fields = columns.iter().map(
                    |&output::RustColumn {
                         ref ident,
                         ref type_,
                         ..
                     }| quote!(#ident: #type_,),
                );

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

                    // .sqlx exists and is a directory, store data.
                    data.save_in(path)?;
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

fn env_db_urls() -> Vec<String> {
    std::env::vars().filter(|(k, _)| k.starts_with("DATABASE_URL")).map(|(_, v)| v).collect()
}
