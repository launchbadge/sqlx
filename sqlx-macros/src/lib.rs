#![cfg_attr(
    not(any(feature = "postgres", feature = "mysql")),
    allow(dead_code, unused_macros, unused_imports)
)]
extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

#[cfg(feature = "runtime-async-std")]
use async_std::task::block_on;

use url::Url;

type Error = Box<dyn std::error::Error>;

type Result<T> = std::result::Result<T, Error>;

mod database;
mod derives;
mod query_macros;
mod runtime;

use query_macros::*;

#[cfg(feature = "runtime-tokio")]
lazy_static::lazy_static! {
    static ref BASIC_RUNTIME: tokio::runtime::Runtime = {
        tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_io()
            .enable_time()
            .build()
            .expect("failed to build tokio runtime")
    };
}

#[cfg(feature = "runtime-tokio")]
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    BASIC_RUNTIME.enter(|| futures::executor::block_on(future))
}

fn macro_result(tokens: proc_macro2::TokenStream) -> TokenStream {
    quote!(
        macro_rules! macro_result {
            ($($args:tt)*) => (#tokens)
        }
    )
    .into()
}

macro_rules! async_macro (
    ($db:ident, $input:ident: $ty:ty => $expr:expr) => {{
        let $input = match syn::parse::<$ty>($input) {
            Ok(input) => input,
            Err(e) => return macro_result(e.to_compile_error()),
        };

        let res: Result<proc_macro2::TokenStream> = block_on(async {
            use sqlx::connection::Connect;

            let db_url = Url::parse(&dotenv::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set")?)?;

            match db_url.scheme() {
                #[cfg(feature = "sqlite")]
                "sqlite" => {
                    let $db = sqlx::sqlite::SqliteConnection::connect(db_url.as_str())
                        .await
                        .map_err(|e| format!("failed to connect to database: {}", e))?;

                    $expr.await
                }
                #[cfg(not(feature = "sqlite"))]
                "sqlite" => Err(format!(
                    "DATABASE_URL {} has the scheme of a SQLite database but the `sqlite` \
                     feature of sqlx was not enabled",
                     db_url
                ).into()),
                #[cfg(feature = "postgres")]
                "postgresql" | "postgres" => {
                    let $db = sqlx::postgres::PgConnection::connect(db_url.as_str())
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
                    let $db = sqlx::mysql::MySqlConnection::connect(db_url.as_str())
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
                    macro_result(parse_err.to_compile_error())
                } else {
                    let msg = e.to_string();
                    macro_result(quote!(compile_error!(#msg)))
                }
            }
        }
    }}
);

#[proc_macro]
#[allow(unused_variables)]
pub fn query(input: TokenStream) -> TokenStream {
    #[allow(unused_variables)]
    async_macro!(db, input: QueryMacroInput => expand_query(input, db, true))
}

#[proc_macro]
#[allow(unused_variables)]
pub fn query_unchecked(input: TokenStream) -> TokenStream {
    #[allow(unused_variables)]
    async_macro!(db, input: QueryMacroInput => expand_query(input, db, false))
}

#[proc_macro]
#[allow(unused_variables)]
pub fn query_file(input: TokenStream) -> TokenStream {
    #[allow(unused_variables)]
    async_macro!(db, input: QueryMacroInput => expand_query_file(input, db, true))
}

#[proc_macro]
#[allow(unused_variables)]
pub fn query_file_unchecked(input: TokenStream) -> TokenStream {
    #[allow(unused_variables)]
    async_macro!(db, input: QueryMacroInput => expand_query_file(input, db, false))
}

#[proc_macro]
#[allow(unused_variables)]
pub fn query_as(input: TokenStream) -> TokenStream {
    #[allow(unused_variables)]
    async_macro!(db, input: QueryAsMacroInput => expand_query_as(input, db, true))
}

#[proc_macro]
#[allow(unused_variables)]
pub fn query_file_as(input: TokenStream) -> TokenStream {
    async_macro!(db, input: QueryAsMacroInput => expand_query_file_as(input, db, true))
}

#[proc_macro]
#[allow(unused_variables)]
pub fn query_as_unchecked(input: TokenStream) -> TokenStream {
    #[allow(unused_variables)]
    async_macro!(db, input: QueryAsMacroInput => expand_query_as(input, db, false))
}

#[proc_macro]
#[allow(unused_variables)]
pub fn query_file_as_unchecked(input: TokenStream) -> TokenStream {
    async_macro!(db, input: QueryAsMacroInput => expand_query_file_as(input, db, false))
}

#[proc_macro_derive(Encode, attributes(sqlx))]
pub fn derive_encode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_encode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Decode, attributes(sqlx))]
pub fn derive_decode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_type_encode_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match derives::expand_derive_from_row(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
