use proc_macro::TokenStream;

use quote::quote;

use sqlx_macros_core::*;

#[cfg(feature = "macros")]
#[proc_macro]
pub fn expand_query(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query::QueryMacroInput);

    match query::expand_input(input, FOSS_DRIVERS) {
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
    match derives::expand_derive_encode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "derive")]
#[proc_macro_derive(Decode, attributes(sqlx))]
pub fn derive_decode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "derive")]
#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_type_encode_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "derive")]
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
    use std::collections::HashMap;
    use syn::{parse_macro_input, Expr, ExprArray, ExprLit, ExprPath, ExprTuple, Lit, LitStr};

    // Extract directory path, handling both direct literals and grouped literals
    fn extract_dir(expr: Option<Expr>) -> LitStr {
        if let Some(Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        })) = expr
        {
            return lit_str;
        } else if let Some(Expr::Group(group)) = expr {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = *group.expr
            {
                return lit_str;
            }
        }
        panic!("Expected a string literal for the directory path.");
    }

    // Extract a `String` value from an expression (either a string literal or a variable)
    fn extract_value(expr: Expr, location: &str) -> String {
        match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) => lit_str.value(),
            Expr::Path(ExprPath { path, .. }) => path.segments.last().unwrap().ident.to_string(),
            _ => panic!("Expected a string literal or a variable in {location}"),
        }
    }

    // Parse substitutions, expecting an array of tuples (String, Expr)
    fn parse_substitutions(expr: Option<Expr>) -> Option<HashMap<String, String>> {
        let Expr::Group(group) = expr? else {
            return None;
        };
        let Expr::Array(ExprArray { elems, .. }) = *group.expr else {
            panic!("Expected an array of tuples (String, Expr).");
        };

        let mut map = HashMap::new();
        for elem in elems {
            let Expr::Tuple(ExprTuple {
                elems: tuple_elems, ..
            }) = elem
            else {
                panic!("Expected a tuple (String, Expr). Got {:#?}", elem);
            };

            let mut tuple_elems = tuple_elems.into_iter();

            let key = extract_value(tuple_elems.next().expect("Missing key in tuple."), "key");
            let value = extract_value(
                tuple_elems.next().expect("Missing value in tuple."),
                "value",
            );
            map.insert(key, value);
        }
        Some(map)
    }

    // Parse input and extract directory and optional parameters
    let exp = parse_macro_input!(input as syn::Expr);
    let (dir, parameters) = match exp {
        Expr::Tuple(ExprTuple { elems, .. }) => {
            let mut elems = elems.into_iter();
            (extract_dir(elems.next()), elems.next())
        }
        Expr::Group(group) => {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = *group.expr
            {
                (lit_str, None)
            } else {
                panic!("Expected a tuple with directory path and optional parameters, or a string literal for the directory path.");
            }
        },
        _ => panic!(
            "Expected a tuple with directory path and optional parameters, or a string literal for the directory path."
        ),
    };

    // Parse substitutions and pass to migration expander
    let substitutions = parse_substitutions(parameters);
    match migrate::expand_migrator_from_lit_dir(dir, substitutions) {
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

    match test_attr::expand(args.into(), input) {
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
