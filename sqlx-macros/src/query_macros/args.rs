use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::Expr;

use sqlx::describe::Describe;

use crate::database::{DatabaseExt, ParamChecking};
use crate::query_macros::QueryMacroInput;

/// Returns a tokenstream which typechecks the arguments passed to the macro
/// and binds them to `DB::Arguments` with the ident `query_args`.
pub fn quote_args<DB: DatabaseExt>(
    input: &QueryMacroInput,
    describe: &Describe<DB>,
) -> crate::Result<TokenStream> {
    let db_path = DB::quotable_path();

    if input.arg_names.is_empty() {
        return Ok(quote! {
            let query_args = <#db_path as sqlx::Database>::Arguments::default();
        });
    }

    let args_check = if DB::PARAM_CHECKING == ParamChecking::Strong {
        let param_types = describe
            .param_types
            .iter()
            .zip(&*input.arg_exprs)
            .map(|(type_, expr)| {
                get_type_override(expr)
                    .or_else(|| {
                        Some(
                            DB::param_type_for_id(type_)?
                                .parse::<proc_macro2::TokenStream>()
                                .unwrap(),
                        )
                    })
                    .ok_or_else(|| format!("unknown type param ID: {}", type_).into())
            })
            .collect::<crate::Result<Vec<_>>>()?;

        let args_ty_cons = input.arg_names.iter().enumerate().map(|(i, expr)| {
            // required or `quote!()` emits it as `Nusize`
            let i = syn::Index::from(i);
            // see src/ty_cons.rs in the main repo for details on this hack
            quote_spanned!( expr.span() => {
                sqlx::ty_cons::TyCons::new(args.#i).lift().ty_cons()
            })
        });

        // we want to make sure it doesn't run
        quote! {
            if false {
                use sqlx::ty_cons::TyConsExt as _;
                let _: (#(#param_types),*,) = (#(#args_ty_cons),*,);
            }
        }
    } else {
        // all we can do is check arity which we did in `QueryMacroInput::describe_validate()`
        TokenStream::new()
    };

    let args = input.arg_names.iter();
    let args_count = input.arg_names.len();
    let arg_indices = (0..args_count).map(|i| syn::Index::from(i));
    let arg_indices_2 = arg_indices.clone();

    Ok(quote! {
        // emit as a tuple first so each expression is only evaluated once
        // these could be separate bindings instead but this is how I decided to write it
        let args = (#(&$#args),*,);
        #args_check
        let mut query_args = <#db_path as sqlx::Database>::Arguments::default();
        query_args.reserve(
            #args_count,
            0 #(+ sqlx::encode::Encode::<#db_path>::size_hint(args.#arg_indices))*
        );
        #(query_args.add(args.#arg_indices_2);)*
    })
}

fn get_type_override(expr: &Expr) -> Option<TokenStream> {
    match expr {
        Expr::Cast(cast) => Some(cast.ty.to_token_stream()),
        Expr::Type(ascription) => Some(ascription.ty.to_token_stream()),
        _ => None,
    }
}
