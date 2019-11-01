extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span;

use quote::quote;

use syn::{parse_macro_input, Expr, ExprLit, Lit, LitStr, Token};
use syn::punctuated::Punctuated;
use syn::parse::{self, Parse, ParseStream};

use sha2::{Sha256, Digest};
use sqlx::Postgres;

use tokio::runtime::Runtime;

use std::error::Error as _;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

struct MacroInput {
    sql: String,
    sql_span: Span,
    args: Vec<Expr>
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?
            .into_iter();

        let sql = match args.next() {
            Some(Expr::Lit(ExprLit { lit: Lit::Str(sql), .. })) => sql,
            Some(other_expr) => return Err(parse::Error::new_spanned(other_expr, "expected string literal")),
            None => return Err(input.error("expected SQL string literal")),
        };

        Ok(
            MacroInput {
                sql: sql.value(),
                sql_span: sql.span(),
                args: args.collect(),
            }
        )
    }
}

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);

    eprintln!("expanding macro");

    match Runtime::new().map_err(Error::from).and_then(|runtime| runtime.block_on(process_sql(input))) {
        Ok(ts) => ts,
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<parse::Error>() {
                return parse_err.to_compile_error().into();
            }

            let msg = e.to_string();
            quote! ( compile_error!(#msg) ).into()
        }
    }
}

async fn process_sql(input: MacroInput) -> Result<TokenStream> {
    let hash = dbg!(hex::encode(&Sha256::digest(input.sql.as_bytes())));

    let conn = sqlx::Connection::<Postgres>::establish("postgresql://postgres@127.0.0.1/sqlx_test")
        .await
        .map_err(|e| format!("failed to connect to database: {}", e))?;

    eprintln!("connection established");

    let prepared = conn.prepare(&hash, &input.sql)
        .await
        .map_err(|e| parse::Error::new(input.sql_span, e))?;

    if input.args.len() != prepared.param_types.len() {
        return Err(parse::Error::new(
            Span::call_site(),
            format!("expected {} parameters, got {}", prepared.param_types.len(), input.args.len())
        ).into());
    }

    Ok(quote! { compile_error!("implementation not finished yet") }.into())
}
