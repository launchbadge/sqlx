use std::path::Path;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::Expr;

use sqlx::describe::Describe;

use crate::database::{DatabaseExt, ParamChecking};
use crate::query_macros::QueryMacroInput;

pub fn quote_args<DB: DatabaseExt>(
    input: &QueryMacroInput,
    describe: &Describe<DB>,
) -> crate::Result<TokenStream> {
    if input.args.is_empty() {
        return Ok(quote! {
            let args = ();
        });
    }

    let args_check = if DB::PARAM_CHECKING == ParamChecking::Strong {
        let param_types = describe
            .param_types
            .iter()
            .zip(&*input.args)
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

        let args_ty_cons = input.args.iter().enumerate().map(|(i, expr)| {
            // required or `quote!()` emits it as `Nusize`
            let i = syn::Index::from(i);
            quote_spanned!( expr.span() => {
                use sqlx::ty_cons::TyConsExt as _;
                sqlx::ty_cons::TyCons::new(&args.#i).ty_cons()
            })
        });

        // we want to make sure it doesn't run
        quote! {
            if false {
                let _: (#(#param_types),*,) = (#(#args_ty_cons),*,);
            }
        }
    } else {
        // all we can do is check arity which we did in `QueryMacroInput::describe_validate()`
        TokenStream::new()
    };

    let args = input.args.iter();

    Ok(quote! {
        let args = (#(#args),*,);
        #args_check
    })
}

fn get_type_override(expr: &Expr) -> Option<TokenStream> {
    match expr {
        Expr::Cast(cast) => Some(cast.ty.to_token_stream()),
        Expr::Type(ascription) => Some(ascription.ty.to_token_stream()),
        _ => None,
    }
}
