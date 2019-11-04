extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span;

use quote::{format_ident, quote, quote_spanned, ToTokens};

use syn::{parse_macro_input, Expr, ExprLit, Lit, LitStr, Token, Type};
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::parse::{self, Parse, ParseStream};

use sha2::{Sha256, Digest};
use sqlx::Postgres;

use tokio::runtime::Runtime;

use std::error::Error as _;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

mod postgres;

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
        Ok(ts) => {
            eprintln!("emitting output: {}", ts);
            ts
        },
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

    let param_types = prepared.param_types.iter().zip(&*input.args).map(|(type_, expr)| {
        get_type_override(expr)
            .or_else(|| postgres::map_param_type_oid(*type_))
            .ok_or_else(|| format!("unknown type OID: {}", type_).into())
    })
        .collect::<Result<Vec<_>>>()?;

    let output_types = prepared.fields.iter().map(|field| {
        postgres::map_output_type_oid(field.type_id)
    })
        .collect::<Result<Vec<_>>>()?;

    let params = input.args.iter();

    let params_ty_cons = input.args.iter().enumerate().map(|(i, expr)| {
        quote_spanned!( expr.span() => { use sqlx::TyConsExt as _; (sqlx::TyCons::new(&params.#i)).ty_cons() })
    });

    let query = &input.sql;

    Ok(
        quote! {{
            use sqlx::TyConsExt as _;

            let params = (#(#params),*,);

            if false {
                let _: (#(#param_types),*,) = (#(#params_ty_cons),*,);
            }

            sqlx::CompiledSql::<_, (#(#output_types),*), sqlx::Postgres> {
                query: #query,
                params,
                output: ::core::marker::PhantomData,
                backend: ::core::marker::PhantomData,
            }
        }}
        .into()
    )
}

fn get_type_override(expr: &Expr) -> Option<proc_macro2::TokenStream> {
    match expr {
        Expr::Cast(cast) => Some(cast.ty.to_token_stream()),
        Expr::Type(ascription) => Some(ascription.ty.to_token_stream()),
        _ => None,
    }
}
