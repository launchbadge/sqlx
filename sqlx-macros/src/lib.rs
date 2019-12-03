#![cfg_attr(not(any(feature = "postgres", feature = "mariadb")), allow(dead_code, unused_macros, unused_imports))]
extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro_hack::proc_macro_hack;

use quote::{quote};

use syn::{
    parse,
    parse_macro_input,
};

use async_std::task;

use url::Url;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

mod backend;

mod query;

macro_rules! with_database(
    ($db:ident => $expr:expr) => {
        async {
            let db_url = Url::parse(&dotenv::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set")?)?;

            match db_url.scheme() {
                #[cfg(feature = "postgres")]
                "postgresql" | "postgres" => {
                    let $db = sqlx::Connection::<sqlx::Postgres>::open(db_url.as_str())
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
                #[cfg(feature = "mariadb")]
                "mysql" | "mariadb" => {
                    let $db = sqlx::Connection::<sqlx::MariaDb>::open(db_url.as_str())
                            .await
                            .map_err(|e| format!("failed to connect to database: {}", e))?;

                    $expr.await
                }
                #[cfg(not(feature = "mariadb"))]
                "mysql" | "mariadb" => Err(format!(
                    "DATABASE_URL {} has the scheme of a MySQL/MariaDB database but the `mariadb` \
                     feature of sqlx was not enabled",
                     db_url
                ).into()),
                scheme => Err(format!("unexpected scheme {:?} in DATABASE_URL {}", scheme, db_url).into()),
            }
        }
    }
);

#[proc_macro_hack]
pub fn query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as query::MacroInput);

    match task::block_on(with_database!(db => query::process_sql(input, db))) {
        Ok(ts) => ts.into(),
        Result::Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<parse::Error>() {
                return parse_err.to_compile_error().into();
            }

            let msg = e.to_string();
            quote!(compile_error!(#msg)).into()
        }
    }
}
