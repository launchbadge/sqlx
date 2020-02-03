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

        let args_ty_checks = input.arg_names.iter().zip(param_types).map(|(arg, ty)| {
            // see src/ty_match in the main repo for details on this hack
            quote_spanned!( arg.span() => {
                let mut arg_check = sqlx::ty_match::InnerType::new(#arg).inner_type();
                arg_check = sqlx::ty_match::MatchBorrow::<#ty, _>::new(arg_check).match_borrow();
            })
        });

        // we want to make sure it doesn't run
        quote! {
            if false {
                use sqlx::ty_match::{InnerTypeExt as _, MatchBorrowExt as _};
                #(#args_ty_checks)*
            }
        }
    } else {
        // all we can do is check arity which we did in `QueryMacroInput::describe_validate()`
        TokenStream::new()
    };

    let arg_name = &input.arg_names;
    let args_count = input.arg_names.len();

    Ok(quote! {
        // bind as a local expression, by-ref
        #(let #arg_name = &$#arg_name;)*
        #args_check
        let mut query_args = <#db_path as sqlx::Database>::Arguments::default();
        query_args.reserve(
            #args_count,
            0 #(+ sqlx::encode::Encode::<#db_path>::size_hint(#arg_name))*
        );
        #(query_args.add(#arg_name);)*
    })
}

fn get_type_override(expr: &Expr) -> Option<TokenStream> {
    match expr {
        Expr::Cast(cast) => Some(cast.ty.to_token_stream()),
        Expr::Type(ascription) => Some(ascription.ty.to_token_stream()),
        _ => None,
    }
}
