extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span;

use proc_macro_hack::proc_macro_hack;

use quote::{quote, quote_spanned, ToTokens};

use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Expr, ExprLit, Lit, Token,
};

use sqlx::HasTypeMetadata;

use async_std::task;

use std::fmt::Display;
use url::Url;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

mod backend;

use backend::BackendExt;

struct MacroInput {
    sql: String,
    sql_span: Span,
    args: Vec<Expr>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?.into_iter();

        let sql = match args.next() {
            Some(Expr::Lit(ExprLit {
                lit: Lit::Str(sql), ..
            })) => sql,
            Some(other_expr) => {
                return Err(parse::Error::new_spanned(
                    other_expr,
                    "expected string literal",
                ));
            }
            None => return Err(input.error("expected SQL string literal")),
        };

        Ok(MacroInput {
            sql: sql.value(),
            sql_span: sql.span(),
            args: args.collect(),
        })
    }
}

#[proc_macro_hack]
pub fn query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);

    match task::block_on(process_sql(input)) {
        Ok(ts) => ts,
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<parse::Error>() {
                return parse_err.to_compile_error().into();
            }

            let msg = e.to_string();
            quote!(compile_error!(#msg)).into()
        }
    }
}

async fn process_sql(input: MacroInput) -> Result<TokenStream> {
    let db_url = Url::parse(&dotenv::var("DATABASE_URL")?)?;

    match db_url.scheme() {
        #[cfg(feature = "postgres")]
        "postgresql" | "postgres" => {
            process_sql_with(
                input,
                sqlx::Connection::<sqlx::Postgres>::open(db_url.as_str())
                    .await
                    .map_err(|e| format!("failed to connect to database: {}", e))?,
            )
            .await
        }
        #[cfg(feature = "mariadb")]
        "mysql" | "mariadb" => {
            process_sql_with(
                input,
                sqlx::Connection::<sqlx::MariaDb>::open(db_url.as_str())
                    .await
                    .map_err(|e| format!("failed to connect to database: {}", e))?,
            )
            .await
        }
        scheme => Err(format!("unexpected scheme {:?} in DB_URL {}", scheme, db_url).into()),
    }
}

async fn process_sql_with<DB: BackendExt>(
    input: MacroInput,
    mut conn: sqlx::Connection<DB>,
) -> Result<TokenStream>
where
    <DB as HasTypeMetadata>::TypeId: Display,
{
    let prepared = conn
        .describe(&input.sql)
        .await
        .map_err(|e| parse::Error::new(input.sql_span, e))?;

    if input.args.len() != prepared.param_types.len() {
        return Err(parse::Error::new(
            Span::call_site(),
            format!(
                "expected {} parameters, got {}",
                prepared.param_types.len(),
                input.args.len()
            ),
        )
        .into());
    }

    let param_types = prepared
        .param_types
        .iter()
        .zip(&*input.args)
        .map(|(type_, expr)| {
            get_type_override(expr)
                .or_else(|| {
                    Some(
                        <DB as BackendExt>::param_type_for_id(type_)?
                            .parse::<proc_macro2::TokenStream>()
                            .unwrap(),
                    )
                })
                .ok_or_else(|| format!("unknown type param ID: {}", type_).into())
        })
        .collect::<Result<Vec<_>>>()?;

    let output_types = prepared
        .result_fields
        .iter()
        .map(|column| {
            Ok(<DB as BackendExt>::return_type_for_id(&column.type_id)
                .ok_or_else(|| format!("unknown field type ID: {}", &column.type_id))?
                .parse::<proc_macro2::TokenStream>()
                .unwrap())
        })
        .collect::<Result<Vec<_>>>()?;

    let params_ty_cons = input.args.iter().enumerate().map(|(i, expr)| {
        // required or `quote!()` emits it as `Nusize`
        let i = syn::Index::from(i);
        quote_spanned!( expr.span() => { use sqlx::TyConsExt as _; (sqlx::TyCons::new(&params.#i)).ty_cons() })
    });

    let query = &input.sql;
    let backend_path = syn::parse_str::<syn::Path>(DB::BACKEND_PATH).unwrap();

    let params = if input.args.is_empty() {
        quote! {
            let params = ();
        }
    } else {
        let params = input.args.iter();

        quote! {
            let params = (#(#params),*,);

            if false {
                use sqlx::TyConsExt as _;

                let _: (#(#param_types),*,) = (#(#params_ty_cons),*,);
            }
        }
    };

    Ok(quote! {{
        #params

        sqlx::Query::<#backend_path, _, (#(#output_types),*,), _> {
            query: #query,
            input: params,
            output: ::core::marker::PhantomData,
            target: ::core::marker::PhantomData,
            backend: ::core::marker::PhantomData,
        }
    }}
    .into())
}

fn get_type_override(expr: &Expr) -> Option<proc_macro2::TokenStream> {
    match expr {
        Expr::Cast(cast) => Some(cast.ty.to_token_stream()),
        Expr::Type(ascription) => Some(ascription.ty.to_token_stream()),
        _ => None,
    }
}
