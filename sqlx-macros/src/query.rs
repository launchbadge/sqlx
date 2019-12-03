use proc_macro2::Span;

use proc_macro2::TokenStream;

use syn::{
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Expr, ExprLit, Lit, Token, Ident,
};

use crate::backend::BackendExt;

use quote::{format_ident, quote, quote_spanned, ToTokens};

use sqlx::{Executor, HasTypeMetadata};

use std::fmt::Display;

pub struct MacroInput {
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

/// Given an input like `query!("SELECT * FROM accounts WHERE account_id > ?", account_id)`
pub async fn process_sql<DB: BackendExt>(
    input: MacroInput,
    mut conn: sqlx::Connection<DB>,
) -> crate::Result<TokenStream>
    where
        <DB as HasTypeMetadata>::TypeId: Display,
{
    let describe = conn
        .describe(&input.sql)
        .await
        .map_err(|e| parse::Error::new(input.sql_span, e))?;

    if input.args.len() != describe.param_types.len() {
        return Err(parse::Error::new(
            Span::call_site(),
            format!(
                "expected {} parameters, got {}",
                describe.param_types.len(),
                input.args.len()
            ),
        )
            .into());
    }

    let param_types = describe
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
        .collect::<crate::Result<Vec<_>>>()?;

    let params_ty_cons = input.args.iter().enumerate().map(|(i, expr)| {
        // required or `quote!()` emits it as `Nusize`
        let i = syn::Index::from(i);
        quote_spanned!( expr.span() => { use sqlx::TyConsExt as _; (sqlx::TyCons::new(&params.#i)).ty_cons() })
    });

    let query = &input.sql;
    let backend_path = DB::quotable_path();

    // record_type will be wrapped in parens which the compiler ignores without a trailing comma
    // e.g. (Foo) == Foo but (Foo,) = one-element tuple
    // and giving an empty stream for record_type makes it unit `()`
    let (record_type, record) = if describe.result_fields.is_empty() {
        (TokenStream::new(), TokenStream::new())
    } else {
        let record_type = Ident::new("Record", Span::call_site());
        (record_type.to_token_stream(), generate_record_def(&describe, &record_type)?)
    };

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
        #record

        #params

        sqlx::Query::<#backend_path, _, (#record_type)> {
            query: #query,
            input: params,
            output: ::core::marker::PhantomData,
            backend: ::core::marker::PhantomData,
        }
    }})
}

fn generate_record_def<DB: BackendExt>(describe: &sqlx::Describe<DB>, type_name: &Ident) -> crate::Result<TokenStream> {
    let fields = describe.result_fields.iter().enumerate()
        .map(|(i, column)| {
            let name = column.name.as_deref()
                .ok_or_else(|| format!("column at position {} must have a name", i))?;

            let name = syn::parse_str::<Ident>(name)
                .map_err(|_| format!("{:?} is not a valid Rust identifier", name))?;

            let type_ = <DB as BackendExt>::return_type_for_id(&column.type_id)
                .ok_or_else(|| format!("unknown field type ID: {}", &column.type_id))?
                .parse::<proc_macro2::TokenStream>()
                .unwrap();

            Ok((name, type_))
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(|e| format!("all SQL result columns must be named with valid Rust identifiers: {}", e))?;

    let row_param = format_ident!("row");

    let record_fields = fields.iter()
        .map(|(name, type_)| quote!(#name: #type_,))
        .collect::<TokenStream>();

    let instantiations = fields.iter()
        .enumerate()
        .map(|(i, (name, _))| quote!(#name: #row_param.get(#i),))
        .collect::<TokenStream>();

    let backend = DB::quotable_path();

    Ok(quote! {
        #[derive(Debug)]
        struct #type_name {
            #record_fields
        }

        impl sqlx::FromRow<#backend> for #type_name {
            fn from_row(#row_param: <#backend as sqlx::Backend>::Row) -> Self {
                use sqlx::Row as _;

                #type_name {
                    #instantiations
                }
            }
        }
    })
}

fn get_type_override(expr: &Expr) -> Option<proc_macro2::TokenStream> {
    match expr {
        Expr::Cast(cast) => Some(cast.ty.to_token_stream()),
        Expr::Type(ascription) => Some(ascription.ty.to_token_stream()),
        _ => None,
    }
}
