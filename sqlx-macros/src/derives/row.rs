use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Expr, Field,
    Fields, FieldsNamed, FieldsUnnamed, Lifetime, Stmt,
};

use super::{
    attributes::{parse_child_attributes, parse_container_attributes},
    rename_all,
};

pub fn expand_derive_from_row(input: &DeriveInput) -> syn::Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => expand_derive_from_row_struct(input, named),

        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) => expand_derive_from_row_struct_unnamed(input, unnamed),

        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => Err(syn::Error::new_spanned(
            input,
            "unit structs are not supported",
        )),

        Data::Enum(_) => Err(syn::Error::new_spanned(input, "enums are not supported")),

        Data::Union(_) => Err(syn::Error::new_spanned(input, "unions are not supported")),
    }
}

fn expand_derive_from_row_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let generics = &input.generics;

    let (lifetime, provided) = generics
        .lifetimes()
        .next()
        .map(|def| (def.lifetime.clone(), false))
        .unwrap_or_else(|| (Lifetime::new("'a", Span::call_site()), true));

    let (_, ty_generics, _) = generics.split_for_impl();

    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(R: ::sqlx::Row));

    if provided {
        generics.params.insert(0, parse_quote!(#lifetime));
    }

    let predicates = &mut generics.make_where_clause().predicates;

    predicates.push(parse_quote!(&#lifetime ::std::primitive::str: ::sqlx::ColumnIndex<R>));

    let container_attributes = parse_container_attributes(&input.attrs)?;

    let reads: Vec<Stmt> = fields
        .iter()
        .filter_map(|field| -> Option<Stmt> {
            let id = &field.ident.as_ref()?;
            let attributes = parse_child_attributes(&field.attrs).unwrap();
            let ty = &field.ty;

            let expr: Expr = match (attributes.flatten, attributes.try_from) {
                (true, None) => {
                    predicates.push(parse_quote!(#ty: ::sqlx::FromRow<#lifetime, R>));
                    parse_quote!(<#ty as ::sqlx::FromRow<#lifetime, R>>::from_row(row))
                }
                (false, None) => {
                    predicates
                        .push(parse_quote!(#ty: ::sqlx::decode::Decode<#lifetime, R::Database>));
                    predicates.push(parse_quote!(#ty: ::sqlx::types::Type<R::Database>));

                    let id_s = attributes
                        .rename
                        .or_else(|| Some(id.to_string().trim_start_matches("r#").to_owned()))
                        .map(|s| match container_attributes.rename_all {
                            Some(pattern) => rename_all(&s, pattern),
                            None => s,
                        })
                        .unwrap();
                    parse_quote!(row.try_get(#id_s))
                }
                (true,Some(try_from)) => {
                    predicates.push(parse_quote!(#try_from: ::sqlx::FromRow<#lifetime, R>));
                    parse_quote!(<#try_from as ::sqlx::FromRow<#lifetime, R>>::from_row(row).and_then(|v| <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v).map_err(|e| ::sqlx::Error::ColumnNotFound("FromRow: try_from failed".to_string())))) 
                }
                (false,Some(try_from)) => {
                    predicates
                        .push(parse_quote!(#try_from: ::sqlx::decode::Decode<#lifetime, R::Database>));
                    predicates.push(parse_quote!(#try_from: ::sqlx::types::Type<R::Database>)); 

                    let id_s = attributes
                        .rename
                        .or_else(|| Some(id.to_string().trim_start_matches("r#").to_owned()))
                        .map(|s| match container_attributes.rename_all {
                            Some(pattern) => rename_all(&s, pattern),
                            None => s,
                        })
                        .unwrap();
                    parse_quote!(row.try_get(#id_s).and_then(|v| <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v).map_err(|e| ::sqlx::Error::ColumnNotFound("FromRow: try_from failed".to_string()))))
                }
            };

            if attributes.default {
                Some(parse_quote!(let #id: #ty = #expr.or_else(|e| match e {
                ::sqlx::Error::ColumnNotFound(_) => {
                    ::std::result::Result::Ok(Default::default())
                },
                e => ::std::result::Result::Err(e)
            })?;))
            } else {
                Some(parse_quote!(
                    let #id: #ty = #expr?;
                ))
            }
        })
        .collect();

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let names = fields.iter().map(|field| &field.ident);

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics ::sqlx::FromRow<#lifetime, R> for #ident #ty_generics #where_clause {
            fn from_row(row: &#lifetime R) -> ::sqlx::Result<Self> {
                #(#reads)*

                ::std::result::Result::Ok(#ident {
                    #(#names),*
                })
            }
        }
    ))
}

fn expand_derive_from_row_struct_unnamed(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let generics = &input.generics;

    let (lifetime, provided) = generics
        .lifetimes()
        .next()
        .map(|def| (def.lifetime.clone(), false))
        .unwrap_or_else(|| (Lifetime::new("'a", Span::call_site()), true));

    let (_, ty_generics, _) = generics.split_for_impl();

    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(R: ::sqlx::Row));

    if provided {
        generics.params.insert(0, parse_quote!(#lifetime));
    }

    let predicates = &mut generics.make_where_clause().predicates;

    predicates.push(parse_quote!(
        ::std::primitive::usize: ::sqlx::ColumnIndex<R>
    ));

    for field in fields {
        let ty = &field.ty;

        predicates.push(parse_quote!(#ty: ::sqlx::decode::Decode<#lifetime, R::Database>));
        predicates.push(parse_quote!(#ty: ::sqlx::types::Type<R::Database>));
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let gets = fields
        .iter()
        .enumerate()
        .map(|(idx, _)| quote!(row.try_get(#idx)?));

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics ::sqlx::FromRow<#lifetime, R> for #ident #ty_generics #where_clause {
            fn from_row(row: &#lifetime R) -> ::sqlx::Result<Self> {
                ::std::result::Result::Ok(#ident (
                    #(#gets),*
                ))
            }
        }
    ))
}
