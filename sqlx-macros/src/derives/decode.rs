use super::attributes::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_attributes,
};
use proc_macro2::Ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Arm, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, Stmt, Variant,
};

pub fn expand_derive_decode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = parse_attributes(&input.attrs)?;
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) if unnamed.len() == 1 => {
            expand_derive_decode_transparent(input, unnamed.first().unwrap())
        }
        Data::Enum(DataEnum { variants, .. }) => match attrs.repr {
            Some(_) => expand_derive_decode_weak_enum(input, variants),
            None => expand_derive_decode_strong_enum(input, variants),
        },
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => expand_derive_decode_struct(input, named),
        _ => Err(syn::Error::new_spanned(
            input,
            "expected a tuple struct with a single field",
        )),
    }
}

fn expand_derive_decode_transparent(
    input: &DeriveInput,
    field: &Field,
) -> syn::Result<proc_macro2::TokenStream> {
    check_transparent_attributes(input, field)?;

    let ident = &input.ident;
    let ty = &field.ty;

    // extract type generics
    let generics = &input.generics;
    let (_, ty_generics, _) = generics.split_for_impl();

    // add db type for impl generics & where clause
    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(DB: sqlx::Database));
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#ty: sqlx::decode::Decode<DB>));
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote!(
        impl #impl_generics sqlx::decode::Decode<DB> for #ident #ty_generics #where_clause {
            fn decode(raw: &[u8]) -> std::result::Result<Self, sqlx::decode::DecodeError> {
                <#ty as sqlx::decode::Decode<DB>>::decode(raw).map(Self)
            }
            fn decode_null() -> std::result::Result<Self, sqlx::decode::DecodeError> {
                <#ty as sqlx::decode::Decode<DB>>::decode_null().map(Self)
            }
            fn decode_nullable(raw: std::option::Option<&[u8]>) -> std::result::Result<Self, sqlx::decode::DecodeError> {
                <#ty as sqlx::decode::Decode<DB>>::decode_nullable(raw).map(Self)
            }
        }
    ))
}

fn expand_derive_decode_weak_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let repr = check_weak_enum_attributes(input, &variants)?;

    let ident = &input.ident;
    let arms = variants
        .iter()
        .map(|v| {
            let id = &v.ident;
            parse_quote!(_ if (#ident :: #id as #repr) == val => Ok(#ident :: #id),)
        })
        .collect::<Vec<Arm>>();

    Ok(quote!(
        impl<DB: sqlx::Database> sqlx::decode::Decode<DB> for #ident where #repr: sqlx::decode::Decode<DB> {
            fn decode(raw: &[u8]) -> std::result::Result<Self, sqlx::decode::DecodeError> {
                let val = <#repr as sqlx::decode::Decode<DB>>::decode(raw)?;
                match val {
                    #(#arms)*
                    _ => Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new("Invalid value")))
                }
            }
        }
    ))
}

fn expand_derive_decode_strong_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    check_strong_enum_attributes(input, &variants)?;

    let ident = &input.ident;

    let mut value_arms = Vec::new();
    for v in variants {
        let id = &v.ident;
        let attributes = parse_attributes(&v.attrs)?;
        if let Some(rename) = attributes.rename {
            value_arms.push(quote!(#rename => Ok(#ident :: #id),));
        } else {
            let name = id.to_string();
            value_arms.push(quote!(#name => Ok(#ident :: #id),));
        }
    }

    // TODO: prevent heap allocation
    Ok(quote!(
        impl<DB: sqlx::Database> sqlx::decode::Decode<DB> for #ident where std::string::String: sqlx::decode::Decode<DB> {
            fn decode(buf: &[u8]) -> std::result::Result<Self, sqlx::decode::DecodeError> {
                let val = <String as sqlx::decode::Decode<DB>>::decode(buf)?;
                match val.as_str() {
                    #(#value_arms)*
                    _ => Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new("Invalid value")))
                }
            }
        }
    ))
}

fn expand_derive_decode_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    check_struct_attributes(input, fields)?;

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "postgres") {
        let ident = &input.ident;

        let column_count = fields.len();

        // extract type generics
        let generics = &input.generics;
        let (_, ty_generics, _) = generics.split_for_impl();

        // add db type for impl generics & where clause
        let mut generics = generics.clone();
        let predicates = &mut generics.make_where_clause().predicates;
        for field in fields {
            let ty = &field.ty;
            predicates.push(parse_quote!(#ty: sqlx::decode::Decode<sqlx::Postgres>));
            predicates.push(parse_quote!(sqlx::Postgres: sqlx::types::HasSqlType<#ty>));
        }
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let mut reads: Vec<Vec<Stmt>> = Vec::new();
        let mut names: Vec<Ident> = Vec::new();
        for field in fields {
            let id = &field.ident;
            names.push(id.clone().unwrap());
            let ty = &field.ty;
            reads.push(parse_quote!(
            if buf.len() < 8 {
                return Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new("Not enough data sent")));
            }

            let oid = u32::from_be_bytes(std::convert::TryInto::try_into(&buf[0..4]).unwrap());
            if oid != <sqlx::Postgres as sqlx::types::HasSqlType<#ty>>::type_info().oid() {
                return Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new("Invalid oid")));
            }

            let len = u32::from_be_bytes(std::convert::TryInto::try_into(&buf[4..8]).unwrap()) as usize;

            if buf.len() < 8 + len {
                return Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new("Not enough data sent")));
            }

            let raw = &buf[8..8+len];
            let #id = <#ty as sqlx::decode::Decode<sqlx::Postgres>>::decode(raw)?;

            let buf = &buf[8+len..];
        ));
        }
        let reads = reads.into_iter().flatten();

        tts.extend(quote!(
        impl #impl_generics sqlx::decode::Decode<sqlx::Postgres> for #ident#ty_generics #where_clause {
            fn decode(buf: &[u8]) -> std::result::Result<Self, sqlx::decode::DecodeError> {
                if buf.len() < 4 {
                    return Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new("Not enough data sent")));
                }

                let column_count = u32::from_be_bytes(std::convert::TryInto::try_into(&buf[..4]).unwrap()) as usize;
                if column_count != #column_count {
                    return Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new("Invalid column count")));
                }
                let buf = &buf[4..];

                #(#reads)*

                if !buf.is_empty() {
                    return Err(sqlx::decode::DecodeError::Message(std::boxed::Box::new(format!("Too much data sent ({} bytes left)", buf.len()))));
                }

                Ok(#ident {
                    #(#names),*
                })
            }
        }
    ))
    }
    Ok(tts)
}
