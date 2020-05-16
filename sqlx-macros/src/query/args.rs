use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::Expr;

use quote::{quote, quote_spanned, ToTokens};
use sqlx_core::describe::Describe;

use crate::database::{DatabaseExt, ParamChecking};
use crate::query::QueryMacroInput;

/// Returns a tokenstream which typechecks the arguments passed to the macro
/// and binds them to `DB::Arguments` with the ident `query_args`.
pub fn quote_args<DB: DatabaseExt>(
    input: &QueryMacroInput,
    describe: &Describe<DB>,
) -> crate::Result<TokenStream> {
    let db_path = DB::db_path();

    if input.arg_names.is_empty() {
        return Ok(quote! {
            let query_args = <#db_path as sqlx::Database>::Arguments::default();
        });
    }

    let arg_name = &input.arg_names;

    let args_check = if input.checked && DB::PARAM_CHECKING == ParamChecking::Strong {
        describe
            .param_types
            .iter()
            .zip(input.arg_names.iter().zip(&input.arg_exprs))
            .enumerate()
            .map(|(i, (param_ty, (name, expr)))| -> crate::Result<_> {
                // TODO: We could remove the ParamChecking flag and just filter to only test params that are non-null
                let param_ty = param_ty.as_ref().unwrap();

                let param_ty = get_type_override(expr)
                    .or_else(|| {
                        Some(
                            DB::param_type_for_id(&param_ty)?
                                .parse::<proc_macro2::TokenStream>()
                                .unwrap(),
                        )
                    })
                    .ok_or_else(|| {
                        if let Some(feature_gate) = <DB as DatabaseExt>::get_feature_gate(&param_ty) {
                            format!(
                                "optional feature `{}` required for type {} of param #{}",
                                feature_gate,
                                param_ty,
                                i + 1,
                            )
                        } else {
                            format!("unsupported type {} for param #{}", param_ty, i + 1)
                        }
                    })?;

                Ok(quote_spanned!(expr.span() =>
                    // this shouldn't actually run
                    if false {
                        use sqlx::ty_match::{WrapSameExt as _, MatchBorrowExt as _};

                        // evaluate the expression only once in case it contains moves
                        let _expr = sqlx::ty_match::dupe_value(&$#name);

                        // if `_expr` is `Option<T>`, get `Option<$ty>`, otherwise `$ty`
                        let ty_check = sqlx::ty_match::WrapSame::<#param_ty, _>::new(&_expr).wrap_same();
                        // if `_expr` is `&str`, convert `String` to `&str`
                        let (mut ty_check, match_borrow) = sqlx::ty_match::MatchBorrow::new(ty_check, &_expr);

                        ty_check = match_borrow.match_borrow();

                        // this causes move-analysis to effectively ignore this block
                        panic!();
                    }
                ))
            })
            .collect::<crate::Result<TokenStream>>()?
    } else {
        // all we can do is check arity which we did in `QueryMacroInput::describe_validate()`
        TokenStream::new()
    };

    let args_count = input.arg_names.len();

    Ok(quote! {
        #args_check

        // bind as a local expression, by-ref
        #(let #arg_name = &$#arg_name;)*
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
