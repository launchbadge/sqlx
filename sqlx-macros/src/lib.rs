extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

use syn::parse_macro_input;

use sha2::{Sha256, Digest};
use sqlx::Postgres;

use tokio::runtime::Runtime;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let string = parse_macro_input!(input as syn::LitStr).value();

    eprintln!("expanding macro");

    match Runtime::new().map_err(Error::from).and_then(|runtime| runtime.block_on(process_sql(&string))) {
        Ok(ts) => ts,
        Err(e) => {
            let msg = e.to_string();
            quote! ( compile_error!(#msg) ).into()
        }
    }
}

async fn process_sql(sql: &str) -> Result<TokenStream> {
    let hash = dbg!(hex::encode(&Sha256::digest(sql.as_bytes())));

    let conn = sqlx::Connection::<Postgres>::establish("postgresql://postgres@127.0.0.1/sqlx_test")
        .await
        .map_err(|e| format!("failed to connect to database: {}", e))?;

    eprintln!("connection established");

    let prepared = conn.prepare(&hash, sql).await?;

    let msg = format!("{:?}", prepared);

    Ok(quote! { compile_error!(#msg) }.into())
}
