use std::fmt::Display;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::{Ident, Path};

use quote::{format_ident, quote};
use sqlx_core::{connection::Connection, database::Database};

use super::{args, output, QueryMacroInput};
use crate::database::DatabaseExt;

/// Given an input like `query!("SELECT * FROM accounts WHERE account_id > ?", account_id)`,
/// expand to an anonymous record
pub async fn expand_query<C: Connection>(
    input: QueryMacroInput,
    mut conn: C,
    checked: bool,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as Database>::TypeInfo: Display,
{
    let describe = input.describe_validate(&mut conn).await?;
    let sql = &input.src;

    let args = args::quote_args(&input, &describe, checked)?;

    let arg_names = &input.arg_names;
    let db_path = <C::Database as DatabaseExt>::db_path();

    if describe.result_columns.is_empty() {
        return Ok(quote! {
            macro_rules! macro_result {
                (#($#arg_names:expr),*) => {{
                    use sqlx_core::arguments::Arguments as _;

                    #args

                    sqlx::query::<#db_path>(#sql).bind_all(query_args)
                }
            }}
        });
    }

    let columns = output::columns_to_rust(&describe)?;

    let record_type: Path = Ident::new("Record", Span::call_site()).into();

    let record_fields = columns
        .iter()
        .map(
            |&output::RustColumn {
                 ref ident,
                 ref type_,
             }| quote!(#ident: #type_,),
        )
        .collect::<TokenStream>();

    let query_args = format_ident!("query_args");
    let output = output::quote_query_as::<C::Database>(
        sql,
        &record_type,
        &query_args,
        if checked { &columns } else { &[] },
        checked,
    );

    Ok(quote! {
        macro_rules! macro_result {
            (#($#arg_names:expr),*) => {{
                use sqlx_core::arguments::Arguments as _;

                #[derive(Debug)]
                struct #record_type {
                    #record_fields
                }

                #args

                #output
            }
        }}
    })
}
