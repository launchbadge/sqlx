use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Path, Type};

use sqlx_core::describe::Describe;

use crate::database::DatabaseExt;

use crate::query_macros::QueryMacroInput;
use std::fmt::{self, Display, Formatter};

pub struct RustColumn {
    pub(super) ident: Ident,
    pub(super) type_: TokenStream,
}

struct DisplayColumn<'a> {
    // zero-based index, converted to 1-based number
    idx: usize,
    name: Option<&'a str>,
}

impl Display for DisplayColumn<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let num = self.idx + 1;

        if let Some(name) = self.name {
            write!(f, "column #{} ({:?})", num, name)
        } else {
            write!(f, "column #{}", num)
        }
    }
}

pub fn columns_to_rust<DB: DatabaseExt>(describe: &Describe<DB>) -> crate::Result<Vec<RustColumn>> {
    describe
        .result_columns
        .iter()
        .enumerate()
        .map(|(i, column)| -> crate::Result<_> {
            let name = column
                .name
                .as_deref()
                .ok_or_else(|| format!("column at position {} must have a name", i))?;

            let ident = parse_ident(name)?;

            let mut type_ = if let Some(type_info) = &column.type_info {
                <DB as DatabaseExt>::return_type_for_id(&type_info).map_or_else(
                    || {
                        let message = if let Some(feature_gate) =
                            <DB as DatabaseExt>::get_feature_gate(&type_info)
                        {
                            format!(
                                "optional feature `{feat}` required for type {ty} of {col}",
                                ty = &type_info,
                                feat = feature_gate,
                                col = DisplayColumn {
                                    idx: i,
                                    name: column.name.as_deref()
                                }
                            )
                        } else {
                            format!(
                                "unsupported type {ty} of {col}",
                                ty = type_info,
                                col = DisplayColumn {
                                    idx: i,
                                    name: column.name.as_deref()
                                }
                            )
                        };
                        syn::Error::new(Span::call_site(), message).to_compile_error()
                    },
                    |t| t.parse().unwrap(),
                )
            } else {
                syn::Error::new(
                    Span::call_site(),
                    format!(
                        "database couldn't tell us the type of {col}; \
                     this can happen for columns that are the result of an expression",
                        col = DisplayColumn {
                            idx: i,
                            name: column.name.as_deref()
                        }
                    ),
                )
                .to_compile_error()
            };

            if !column.non_null.unwrap_or(false) {
                type_ = quote! { Option<#type_> };
            }

            Ok(RustColumn { ident, type_ })
        })
        .collect::<crate::Result<Vec<_>>>()
}

pub fn quote_query_as<DB: DatabaseExt>(
    input: &QueryMacroInput,
    out_ty: &Type,
    bind_args: &Ident,
    columns: &[RustColumn],
) -> TokenStream {
    let instantiations = columns.iter().enumerate().map(
        |(
            i,
            &RustColumn {
                ref ident,
                ref type_,
                ..
            },
        )| {
            // For "checked" queries, the macro checks these at compile time and using "try_get"
            // would also perform pointless runtime checks

            if input.checked {
                quote!( #ident: row.try_get_unchecked::<#type_, _>(#i).try_unwrap_optional()? )
            } else {
                quote!( #ident: row.try_get_unchecked(#i)? )
            }
        },
    );

    let db_path = DB::db_path();
    let row_path = DB::row_path();
    let sql = &input.src;

    quote! {
        sqlx::query::<#db_path>(#sql).bind_all(#bind_args).try_map(|row: #row_path| {
            use sqlx::Row as _;
            use sqlx::result_ext::ResultExt as _;

            Ok(#out_ty { #(#instantiations),* })
        })
    }
}

fn parse_ident(name: &str) -> crate::Result<Ident> {
    // workaround for the following issue (it's semi-fixed but still spits out extra diagnostics)
    // https://github.com/dtolnay/syn/issues/749#issuecomment-575451318

    let is_valid_ident = name.chars().all(|c| c.is_alphanumeric() || c == '_');

    if is_valid_ident {
        let ident = String::from("r#") + name;
        if let Ok(ident) = syn::parse_str(&ident) {
            return Ok(ident);
        }
    }

    Err(format!("{:?} is not a valid Rust identifier", name).into())
}
