#![cfg_attr(
    not(any(feature = "postgres", feature = "mysql", feature = "offline")),
    allow(dead_code, unused_macros, unused_imports)
)]
extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

type Error = Box<dyn std::error::Error>;

type Result<T> = std::result::Result<T, Error>;

mod common;
mod database;
mod derives;
mod query;

#[cfg(feature = "migrate")]
mod migrate;

#[proc_macro]
pub fn expand_query(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query::QueryMacroInput);

    match query::expand_input(input) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(compile_error!(#msg)).into()
            }
        }
    }
}

#[proc_macro_derive(Encode, attributes(sqlx))]
pub fn derive_encode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_encode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Decode, attributes(sqlx))]
pub fn derive_decode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_type_encode_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match derives::expand_derive_from_row(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "migrate")]
#[proc_macro]
pub fn migrate(input: TokenStream) -> TokenStream {
    use syn::LitStr;

    let input = syn::parse_macro_input!(input as LitStr);
    match migrate::expand_migrator_from_dir(input) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(compile_error!(#msg)).into()
            }
        }
    }
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn test(meta: TokenStream, input: TokenStream) -> TokenStream {
    macro_rules! err_spanned (
        ($tokens:expr, $msg:expr) => (
            return syn::Error::new_spanned($tokens, $msg)
                .to_compile_error()
                .into()
        )
    );

    let input = syn::parse_macro_input!(input as syn::ItemFn);

    let cancelable = if !meta.is_empty() {
        let ident = syn::parse_macro_input!(meta as syn::Ident);

        if ident != "cancelable" {
            err_spanned!(ident, "expected `cancelable` or nothing");
        }

        true
    } else {
        false
    };

    if input.sig.asyncness.is_none() {
        err_spanned!(input.sig.fn_token, "expected `async fn`");
    }

    if cancelable && input.sig.inputs.is_empty() {
        err_spanned!(
            input.sig,
            "in order to test cancellation this function must accept a mutable reference to a connection"
        );
    }

    if input.sig.inputs.len() > 1 {
        err_spanned!(input.sig, "test functions may have *at most* one argument")
    }

    let conn_arg = if let Some(arg) = input.sig.inputs.first() {
        match arg {
            syn::FnArg::Receiver(recv) => err_spanned!(recv, "test functions may not take `self`"),
            syn::FnArg::Typed(pat_ty) => Some(pat_ty),
        }
    } else {
        None
    };

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let block = &input.block;
    let attrs = &input.attrs;

    // expression that connects to the database
    let (connect, bind_conn) = if let Some(conn_arg) = conn_arg {
        let ty = &conn_arg.ty;
        let pat = &conn_arg.pat;

        if let syn::Type::Reference(syn::TypeReference {
            mutability: Some(_),
            elem,
            ..
        }) = &**ty
        {
            (
                quote! {
                    let mut conn = sqlx_test::connect::<#elem>().await?;
                },
                // these are separate so the `test_cancellation()` tokens below can use `conn`
                quote! {
                    let #pat = &mut conn;
                },
            )
        } else {
            err_spanned!(ty, "expected `&mut <PgConnection | MySqlConnection | ...>`")
        }
    } else {
        (quote! {}, quote! {})
    };

    // override that switches to `test_cancellation()` instead of just running the text
    let test_cancel = if cancelable {
        let conn_arg = conn_arg.expect("BUG: conn_arg should have been checked above");

        let inner_name = quote::format_ident!("{}_test_cancellation", input.sig.ident);

        quote! {
            if std::env::var("SQLX_TEST_CANCELLATION").map_or(false, |var| var != "0") {
                // useful so we can hint the return type of the block
                async fn #inner_name(#conn_arg) #ret {
                    #block
                }

                return sqlx_test::test_cancellation(&mut conn, #inner_name).await;
            }
        }
    } else {
        quote! {}
    };

    let body = quote! {
        #connect
        #test_cancel
        #bind_conn

        #block
    };

    let result = if cfg!(feature = "runtime-actix") {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::actix_rt::System::new("sqlx-test")
                    .block_on(async { #body })
            }
        }
    } else {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::block_on(async { #body })
            }
        }
    };

    result.into()
}
