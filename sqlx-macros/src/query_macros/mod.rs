use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub use input::{QueryAsMacroInput, QueryMacroInput};
pub use query::expand_query;

use crate::database::DatabaseExt;

use sqlx::Connection;
use sqlx::Database;

mod args;
mod input;
mod output;
mod query;

pub async fn expand_query_file<C: Connection>(
    input: QueryMacroInput,
    conn: C,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as Database>::TypeInfo: Display,
{
    expand_query(input.expand_file_src().await?, conn).await
}

pub async fn expand_query_as<C: Connection>(
    input: QueryAsMacroInput,
    mut conn: C,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as Database>::TypeInfo: Display,
{
    let describe = input.query_input.describe_validate(&mut conn).await?;

    if describe.result_columns.is_empty() {
        return Err(syn::Error::new(
            input.query_input.source_span,
            "query must output at least one column",
        )
        .into());
    }

    let args_tokens = args::quote_args(&input.query_input, &describe)?;
    let arg_names = &input.query_input.arg_names;

    let query_args = format_ident!("query_args");

    let columns = output::columns_to_rust(&describe)?;
    let output = output::quote_query_as::<C::Database>(
        &input.query_input.source,
        &input.as_ty.path,
        &query_args,
        &columns,
    );

    let db_path = <C::Database as DatabaseExt>::db_path();
    let args_count = arg_names.len();
    let arg_indices = (0..args_count).map(|i| syn::Index::from(i));
    let arg_indices_2 = arg_indices.clone();

    Ok(quote! {
        macro_rules! macro_result {
            (#($#arg_names:expr),*) => {{
                use sqlx::arguments::Arguments as _;

                #args_tokens

                let mut #query_args = <#db_path as sqlx::Database>::Arguments::default();
                #query_args.reserve(
                    #args_count,
                    0 #(+ sqlx::encode::Encode::<#db_path>::size_hint(args.#arg_indices))*
                );
                #(#query_args.add(args.#arg_indices_2);)*

                #output
            }}
        }
    })
}

pub async fn expand_query_file_as<C: Connection>(
    input: QueryAsMacroInput,
    conn: C,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as Database>::TypeInfo: Display,
{
    expand_query_as(input.expand_file_src().await?, conn).await
}
