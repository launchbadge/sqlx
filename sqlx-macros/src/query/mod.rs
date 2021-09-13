use std::path::PathBuf;
#[cfg(feature = "offline")]
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use syn::Type;
use url::Url;

pub use input::QueryMacroInput;
use quote::{format_ident, quote};
use sqlx_core::connection::Connection;
use sqlx_core::database::Database;
use sqlx_core::{column::Column, describe::Describe, type_info::TypeInfo};
use sqlx_rt::block_on;

use crate::database::DatabaseExt;
use crate::query::data::QueryData;
use crate::query::input::RecordType;
use either::Either;

mod args;
mod data;
mod input;
mod output;

struct Metadata {
    #[allow(unused)]
    manifest_dir: PathBuf,
    offline: bool,
    database_url: Option<String>,
    #[cfg(feature = "offline")]
    target_dir: PathBuf,
    #[cfg(feature = "offline")]
    workspace_root: Arc<Mutex<Option<PathBuf>>>,
}

#[cfg(feature = "offline")]
impl Metadata {
    pub fn workspace_root(&self) -> PathBuf {
        let mut root = self.workspace_root.lock().unwrap();
        if root.is_none() {
            use serde::Deserialize;
            use std::process::Command;

            let cargo = env("CARGO").expect("`CARGO` must be set");

            let output = Command::new(&cargo)
                .args(&["metadata", "--format-version=1"])
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

    #[cfg(feature = "offline")]
    let target_dir = env("CARGO_TARGET_DIR").map_or_else(|_| "target".into(), |dir| dir.into());

    // If a .env file exists at CARGO_MANIFEST_DIR, load environment variables from this,
    // otherwise fallback to default dotenv behaviour.
    let env_path = manifest_dir.join(".env");

    #[cfg_attr(not(procmacro2_semver_exempt), allow(unused_variables))]
    let env_path = if env_path.exists() {
        let res = dotenv::from_path(&env_path);
        if let Err(e) = res {
            panic!("failed to load environment from {:?}, {}", env_path, e);
        }

        Some(env_path)
    } else {
        dotenv::dotenv().ok()
    };

    // tell the compiler to watch the `.env` for changes, if applicable
    #[cfg(procmacro2_semver_exempt)]
    if let Some(env_path) = env_path.as_ref().and_then(|path| path.to_str()) {
        proc_macro::tracked_path::path(env_path);
    }

    let offline = env("SQLX_OFFLINE")
        .map(|s| s.eq_ignore_ascii_case("true") || s == "1")
        .unwrap_or(false);

    let database_url = env("DATABASE_URL").ok();

    Metadata {
        manifest_dir,
        offline,
        database_url,
        #[cfg(feature = "offline")]
        target_dir,
        #[cfg(feature = "offline")]
        workspace_root: Arc::new(Mutex::new(None)),
    }
});

pub fn expand_input(input: QueryMacroInput) -> crate::Result<TokenStream> {
    match &*METADATA {
        Metadata {
            offline: false,
            database_url: Some(db_url),
            ..
        } => expand_from_db(input, &db_url),

        #[cfg(feature = "offline")]
        _ => {
            let data_file_path = METADATA.manifest_dir.join("sqlx-data.json");

            if data_file_path.exists() {
                expand_from_file(input, data_file_path)
            } else {
                let workspace_data_file_path = METADATA.workspace_root().join("sqlx-data.json");
                if workspace_data_file_path.exists() {
                    expand_from_file(input, workspace_data_file_path)
                } else {
                    Err(
                        "`DATABASE_URL` must be set, or `cargo sqlx prepare` must have been run \
                     and sqlx-data.json must exist, to use query macros"
                            .into(),
                    )
                }
            }
        }

        #[cfg(not(feature = "offline"))]
        Metadata { offline: true, .. } => {
            Err("The cargo feature `offline` has to be enabled to use `SQLX_OFFLINE`".into())
        }

        #[cfg(not(feature = "offline"))]
        Metadata {
            offline: false,
            database_url: None,
            ..
        } => Err("`DATABASE_URL` must be set to use query macros".into()),
    }
}

#[allow(unused_variables)]
fn expand_from_db(input: QueryMacroInput, db_url: &str) -> crate::Result<TokenStream> {
    // FIXME: Introduce [sqlx::any::AnyConnection] and [sqlx::any::AnyDatabase] to support
    //        runtime determinism here

    let db_url = Url::parse(db_url)?;
    match db_url.scheme() {
        #[cfg(feature = "postgres")]
        "postgres" | "postgresql" => {
            let data = block_on(async {
                let mut conn = sqlx_core::postgres::PgConnection::connect(db_url.as_str()).await?;
                QueryData::from_db(&mut conn, &input.sql).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "postgres"))]
        "postgres" | "postgresql" => Err("database URL has the scheme of a PostgreSQL database but the `postgres` feature is not enabled".into()),

        #[cfg(feature = "mssql")]
        "mssql" | "sqlserver" => {
            let data = block_on(async {
                let mut conn = sqlx_core::mssql::MssqlConnection::connect(db_url.as_str()).await?;
                QueryData::from_db(&mut conn, &input.sql).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "mssql"))]
        "mssql" | "sqlserver" => Err("database URL has the scheme of a MSSQL database but the `mssql` feature is not enabled".into()),

        #[cfg(feature = "mysql")]
        "mysql" | "mariadb" => {
            let data = block_on(async {
                let mut conn = sqlx_core::mysql::MySqlConnection::connect(db_url.as_str()).await?;
                QueryData::from_db(&mut conn, &input.sql).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "mysql"))]
        "mysql" | "mariadb" => Err("database URL has the scheme of a MySQL/MariaDB database but the `mysql` feature is not enabled".into()),

        #[cfg(feature = "sqlite")]
        "sqlite" => {
            let data = block_on(async {
                let mut conn = sqlx_core::sqlite::SqliteConnection::connect(db_url.as_str()).await?;
                QueryData::from_db(&mut conn, &input.sql).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "sqlite"))]
        "sqlite" => Err("database URL has the scheme of a SQLite database but the `sqlite` feature is not enabled".into()),

        scheme => Err(format!("unknown database URL scheme {:?}", scheme).into())
    }
}

#[cfg(feature = "offline")]
pub fn expand_from_file(input: QueryMacroInput, file: PathBuf) -> crate::Result<TokenStream> {
    use data::offline::DynQueryData;

    let query_data = DynQueryData::from_data_file(file, &input.sql)?;
    assert!(!query_data.db_name.is_empty());

    match &*query_data.db_name {
        #[cfg(feature = "postgres")]
        sqlx_core::postgres::Postgres::NAME => expand_with_data(
            input,
            QueryData::<sqlx_core::postgres::Postgres>::from_dyn_data(query_data)?,
            true,
        ),
        #[cfg(feature = "mysql")]
        sqlx_core::mysql::MySql::NAME => expand_with_data(
            input,
            QueryData::<sqlx_core::mysql::MySql>::from_dyn_data(query_data)?,
            true,
        ),
        #[cfg(feature = "sqlite")]
        sqlx_core::sqlite::Sqlite::NAME => expand_with_data(
            input,
            QueryData::<sqlx_core::sqlite::Sqlite>::from_dyn_data(query_data)?,
            true,
        ),
        _ => Err(format!(
            "found query data for {} but the feature for that database was not enabled",
            query_data.db_name
        )
        .into()),
    }
}

// marker trait for `Describe` that lets us conditionally require it to be `Serialize + Deserialize`
#[cfg(feature = "offline")]
trait DescribeExt: serde::Serialize + serde::de::DeserializeOwned {}

#[cfg(feature = "offline")]
impl<DB: Database> DescribeExt for Describe<DB> where
    Describe<DB>: serde::Serialize + serde::de::DeserializeOwned
{
}

#[cfg(not(feature = "offline"))]
trait DescribeExt {}

#[cfg(not(feature = "offline"))]
impl<DB: Database> DescribeExt for Describe<DB> {}

fn expand_with_data<DB: DatabaseExt>(
    input: QueryMacroInput,
    data: QueryData<DB>,
    #[allow(unused_variables)] offline: bool,
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
            ::sqlx::query_with::<#db_path, _>(#sql, #query_args)
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
    #[cfg(feature = "offline")]
    if !offline {
        let save_dir = METADATA.target_dir.join("sqlx");
        std::fs::create_dir_all(&save_dir)?;
        data.save_in(save_dir, input.src_span)?;
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
