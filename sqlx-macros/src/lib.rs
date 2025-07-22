use proc_macro::TokenStream;

use quote::quote;

use sqlx_macros_core::*;

/// Constant used in all macros to define the macros namespace.
/// This accommodates 3rd party drivers by allowing them to specify a different
/// root crate that paths used with proc macros resolve to.
#[cfg(not(test))]
const CRATE_NAME: &str = "sqlx";
// Allows for easier testing of the configurable macros namespace feature
// of current proc macros without duplicating them.
#[cfg(test)]
const CRATE_NAME: &str = env!("SQLX_NAMESPACE");

#[cfg(feature = "macros")]
#[proc_macro]
pub fn expand_query(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query::QueryMacroInput);

    match query::expand_input(input, FOSS_DRIVERS, quote::format_ident!("{CRATE_NAME}")) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

#[cfg(feature = "derive")]
#[proc_macro_derive(Encode, attributes(sqlx))]
pub fn derive_encode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_encode(&input, &quote::format_ident!("{CRATE_NAME}")) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "derive")]
#[proc_macro_derive(Decode, attributes(sqlx))]
pub fn derive_decode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_decode(&input, &quote::format_ident!("{CRATE_NAME}")) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "derive")]
#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_type_encode_decode(&input, quote::format_ident!("{CRATE_NAME}")) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "derive")]
#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match derives::expand_derive_from_row(&input, quote::format_ident!("{CRATE_NAME}")) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "migrate")]
#[proc_macro]
pub fn migrate(input: TokenStream) -> TokenStream {
    use syn::LitStr;

    let input = syn::parse_macro_input!(input as Option<LitStr>);
    match migrate::expand(input, &quote::format_ident!("{CRATE_NAME}")) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

#[cfg(feature = "macros")]
#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);

    match test_attr::expand(args.into(), input, quote::format_ident!("{CRATE_NAME}")) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "migrate")]
    fn test_macros_namespace_migrate() {
        /// Import as different namespace.
        ///
        /// This must be set as `SQLX_NAMESPACE` environment variable to test that
        /// changing the namespace still results in the proc macros behaving well.
        extern crate sqlx as external;

        let _ = external::migrate!("../tests/migrate/migrations_simple");
    }

    #[test]
    #[cfg(feature = "derive")]
    fn test_macros_namespace_derive() {
        /// Import as different namespace.
        ///
        /// This must be set as `SQLX_NAMESPACE` environment variable to test that
        /// changing the namespace still results in the proc macros behaving well.
        extern crate sqlx as external;

        #[derive(Debug, external::Type, external::Encode, external::Decode, external::FromRow)]
        struct Test {}
    }
}
