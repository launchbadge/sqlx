#![cfg_attr(
    not(any(feature = "postgres", feature = "mysql")),
    allow(dead_code, unused_macros, unused_imports)
)]
extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro_hack::proc_macro_hack;

use quote::quote;

use syn::parse_macro_input;

use async_std::task;

use url::Url;

type Error = Box<dyn std::error::Error>;

type Result<T> = std::result::Result<T, Error>;

mod database;

mod query_macros;

use query_macros::*;

macro_rules! async_macro (
    ($db:ident => $expr:expr) => {{
        let res: Result<proc_macro2::TokenStream> = task::block_on(async {
            use sqlx::Connection;

            let db_url = Url::parse(&dotenv::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set")?)?;

            match db_url.scheme() {
                #[cfg(feature = "postgres")]
                "postgresql" | "postgres" => {
                    let $db = sqlx::postgres::PgConnection::open(db_url.as_str())
                        .await
                        .map_err(|e| format!("failed to connect to database: {}", e))?;

                    $expr.await
                }
                #[cfg(not(feature = "postgres"))]
                "postgresql" | "postgres" => Err(format!(
                    "DATABASE_URL {} has the scheme of a Postgres database but the `postgres` \
                     feature of sqlx was not enabled",
                     db_url
                ).into()),
                #[cfg(feature = "mysql")]
                "mysql" | "mariadb" => {
                    let $db = sqlx::mysql::MySqlConnection::open(db_url.as_str())
                            .await
                            .map_err(|e| format!("failed to connect to database: {}", e))?;

                    $expr.await
                }
                #[cfg(not(feature = "mysql"))]
                "mysql" | "mariadb" => Err(format!(
                    "DATABASE_URL {} has the scheme of a MySQL/MariaDB database but the `mysql` \
                     feature of sqlx was not enabled",
                     db_url
                ).into()),
                scheme => Err(format!("unexpected scheme {:?} in DATABASE_URL {}", scheme, db_url).into()),
            }
        });

        match res {
            Ok(ts) => ts.into(),
            Err(e) => {
                if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                    return dbg!(parse_err).to_compile_error().into();
                }

                let msg = format!("{:?}", e);
                quote!(compile_error!(#msg);).into()
            }
        }
    }}
);

#[proc_macro_hack]
pub fn query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryMacroInput);
    async_macro!(db => expand_query(input, db))
}

#[proc_macro_hack]
pub fn query_file(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryMacroInput);
    async_macro!(db => expand_query_file(input, db))
}

#[proc_macro_hack]
pub fn query_as(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryAsMacroInput);
    async_macro!(db => expand_query_as(input, db))
}

#[proc_macro_hack]
pub fn query_file_as(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryAsMacroInput);
    async_macro!(db => expand_query_file_as(input, db))
}
