use std::borrow::Cow;
use std::env;

use proc_macro2::{Span, TokenStream};
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

pub fn expand_input(input: QueryMacroInput) -> crate::Result<TokenStream> {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").map_err(|_| "`CARGO_MANIFEST_DIR` must be set")?;

    // If a .env file exists at CARGO_MANIFEST_DIR, load environment variables from this,
    // otherwise fallback to default dotenv behaviour.
    let env_path = std::path::Path::new(&manifest_dir).join(".env");
    if env_path.exists() {
        dotenv::from_path(&env_path)
            .map_err(|e| format!("failed to load environment from {:?}, {}", env_path, e))?
    }

    // if `dotenv` wasn't initialized by the above we make sure to do it here
    match (
        dotenv::var("SQLX_OFFLINE")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false),
        dotenv::var("DATABASE_URL"),
    ) {
        (false, Ok(db_url)) => expand_from_db(input, &db_url),

        #[cfg(feature = "offline")]
        _ => {
            let data_file_path = std::path::Path::new(&manifest_dir).join("sqlx-data.json");

            if data_file_path.exists() {
                expand_from_file(input, data_file_path)
            } else {
                Err(
                    "`DATABASE_URL` must be set, or `cargo sqlx prepare` must have been run \
                     and sqlx-data.json must exist, to use query macros"
                        .into(),
                )
            }
        }

        #[cfg(not(feature = "offline"))]
        (true, _) => {
            Err("The cargo feature `offline` has to be enabled to use `SQLX_OFFLINE`".into())
        }

        #[cfg(not(feature = "offline"))]
        (false, Err(_)) => Err("`DATABASE_URL` must be set to use query macros".into()),
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
                QueryData::from_db(&mut conn, &input.src).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "postgres"))]
        "postgres" | "postgresql" => Err(format!("database URL has the scheme of a PostgreSQL database but the `postgres` feature is not enabled").into()),

        #[cfg(feature = "mssql")]
        "mssql" | "sqlserver" => {
            let data = block_on(async {
                let mut conn = sqlx_core::mssql::MssqlConnection::connect(db_url.as_str()).await?;
                QueryData::from_db(&mut conn, &input.src).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "mssql"))]
        "mssql" | "sqlserver" => Err(format!("database URL has the scheme of a MSSQL database but the `mssql` feature is not enabled").into()),

        #[cfg(feature = "mysql")]
        "mysql" | "mariadb" => {
            let data = block_on(async {
                let mut conn = sqlx_core::mysql::MySqlConnection::connect(db_url.as_str()).await?;
                QueryData::from_db(&mut conn, &input.src).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "mysql"))]
        "mysql" | "mariadb" => Err(format!("database URL has the scheme of a MySQL/MariaDB database but the `mysql` feature is not enabled").into()),

        #[cfg(feature = "sqlite")]
        "sqlite" => {
            let data = block_on(async {
                let mut conn = sqlx_core::sqlite::SqliteConnection::connect(db_url.as_str()).await?;
                QueryData::from_db(&mut conn, &input.src).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "sqlite"))]
        "sqlite" => Err(format!("database URL has the scheme of a SQLite database but the `sqlite` feature is not enabled").into()),

        scheme => Err(format!("unknown database URL scheme {:?}", scheme).into())
    }
}

#[cfg(feature = "offline")]
pub fn expand_from_file(
    input: QueryMacroInput,
    file: std::path::PathBuf,
) -> crate::Result<TokenStream> {
    use data::offline::DynQueryData;

    let query_data = DynQueryData::from_data_file(file, &input.src)?;
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
            return Err(syn::Error::new(
                Span::call_site(),
                format!("expected {} parameters, got {}", num, input.arg_exprs.len()),
            )
            .into());
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
        let sql = &input.src;

        quote! {
            sqlx::query_with::<#db_path, _>(#sql, #query_args)
        }
    } else {
        let columns = output::columns_to_rust::<DB>(&data.describe)?;

        let (out_ty, mut record_tokens) = match input.record_type {
            RecordType::Generated => {
                let record_name: Type = syn::parse_str("Record").unwrap();

                for rust_col in &columns {
                    if rust_col.type_.is_wildcard() {
                        return Err(
                            "columns may not have wildcard overrides in `query!()` or `query_as!()"
                                .into(),
                        );
                    }
                }

                let record_fields = columns.iter().map(
                    |&output::RustColumn {
                         ref ident,
                         ref type_,
                     }| quote!(#ident: #type_,),
                );

                let record_tokens = quote! {
                    #[derive(Debug)]
                    struct #record_name {
                        #(#record_fields)*
                    }
                };

                (Cow::Owned(record_name), record_tokens)
            }
            RecordType::Given(ref out_ty) => (Cow::Borrowed(out_ty), quote!()),
        };

        record_tokens.extend(output::quote_query_as::<DB>(
            &input,
            &out_ty,
            &query_args,
            &columns,
        ));

        record_tokens
    };

    let ret_tokens = quote! {{
        use sqlx::Arguments as _;

        #args_tokens

        #output
    }};

    // Store query metadata only if offline support is enabled but the current build is online.
    // If the build is offline, the cache is our input so it's pointless to also write data for it.
    #[cfg(feature = "offline")]
    if !offline {
        let mut save_dir = std::path::PathBuf::from(
            env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target/".into()),
        );

        save_dir.push("sqlx");

        std::fs::create_dir_all(&save_dir)?;
        data.save_in(save_dir, input.src_span)?;
    }

    Ok(ret_tokens)
}
