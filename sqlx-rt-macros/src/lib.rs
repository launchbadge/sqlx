use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    let result = if cfg!(feature = "runtime-tokio") {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::tokio::runtime::Builder::new()
                    .threaded_scheduler()
                    .enable_io()
                    .enable_time()
                    .build()
                    .unwrap()
                    .block_on(async { #body })
            }
        }
    } else if cfg!(feature = "runtime-async-std") {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::async_std::task::block_on(async { #body })
            }
        }
    } else if cfg!(feature = "runtime-actix") {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::actix_rt::System::new("sqlx-test")
                    .block_on(async { #body })
            }
        }
    } else {
        panic!("one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features must be enabled");
    };

    result.into()
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    let result = if cfg!(feature = "runtime-tokio") {
        quote! {
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::tokio::runtime::Builder::new()
                    .basic_scheduler()
                    .enable_io()
                    .enable_time()
                    .build()
                    .unwrap()
                    .block_on(async { #body })
            }
        }
    } else if cfg!(feature = "runtime-async-std") {
        quote! {
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::async_std::task::block_on(async { #body })
            }
        }
    } else if cfg!(feature = "runtime-actix") {
        quote! {
            #(#attrs)*
            fn #name() #ret {
                sqlx_rt::actix_rt::System::new("sqlx")
                    .block_on(async { #body })
            }
        }
    } else {
        panic!("one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features must be enabled");
    };

    result.into()
}
