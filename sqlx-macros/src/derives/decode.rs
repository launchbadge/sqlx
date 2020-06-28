use super::attributes::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_child_attributes, parse_container_attributes,
};
use super::rename_all;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Arm, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, Stmt, Variant,
};

pub fn expand_derive_decode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = parse_container_attributes(&input.attrs)?;
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
        Data::Union(_) => Err(syn::Error::new_spanned(input, "unions are not supported")),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(..),
            ..
        }) => Err(syn::Error::new_spanned(
            input,
            "structs with zero or more than one unnamed field are not supported",
        )),
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => Err(syn::Error::new_spanned(
            input,
            "unit structs are not supported",
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
    generics.params.insert(0, parse_quote!('r));
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#ty: sqlx::decode::Decode<'r, DB>));
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let tts = quote!(
        impl #impl_generics sqlx::decode::Decode<'r, DB> for #ident #ty_generics #where_clause {
            fn decode(value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                <#ty as sqlx::decode::Decode<'r, DB>>::decode(value).map(Self)
            }
        }
    );

    Ok(tts)
}

fn expand_derive_decode_weak_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attr = check_weak_enum_attributes(input, &variants)?;
    let repr = attr.repr.unwrap();

    let ident = &input.ident;
    let ident_s = ident.to_string();

    let arms = variants
        .iter()
        .map(|v| {
            let id = &v.ident;
            parse_quote!(_ if (#ident :: #id as #repr) == value => Ok(#ident :: #id),)
        })
        .collect::<Vec<Arm>>();

    Ok(quote!(
        impl<'r, DB: sqlx::Database> sqlx::decode::Decode<'r, DB> for #ident where #repr: sqlx::decode::Decode<'r, DB> {
            fn decode(value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                let value = <#repr as sqlx::decode::Decode<'r, DB>>::decode(value)?;

                match value {
                    #(#arms)*

                    _ => Err(Box::new(sqlx::Error::Decode(format!("invalid value {:?} for enum {}", value, #ident_s).into())))
                }
            }
        }
    ))
}

fn expand_derive_decode_strong_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let cattr = check_strong_enum_attributes(input, &variants)?;

    let ident = &input.ident;
    let ident_s = ident.to_string();

    let value_arms = variants.iter().map(|v| -> Arm {
        let id = &v.ident;
        let attributes = parse_child_attributes(&v.attrs).unwrap();

        if let Some(rename) = attributes.rename {
            parse_quote!(#rename => Ok(#ident :: #id),)
        } else if let Some(pattern) = cattr.rename_all {
            let name = rename_all(&*id.to_string(), pattern);

            parse_quote!(#name => Ok(#ident :: #id),)
        } else {
            let name = id.to_string();
            parse_quote!(#name => Ok(#ident :: #id),)
        }
    });

    let values = quote! {
        match value {
            #(#value_arms)*

            _ => Err(format!("invalid value {:?} for enum {}", value, #ident_s).into())
        }
    };

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl<'r> sqlx::decode::Decode<'r, sqlx::mysql::MySql> for #ident {
                fn decode(value: sqlx::mysql::MySqlValueRef<'r>) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                    let value = <&'r str as sqlx::decode::Decode<'r, sqlx::mysql::MySql>>::decode(value)?;

                    #values
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        tts.extend(quote!(
            impl<'r> sqlx::decode::Decode<'r, sqlx::postgres::Postgres> for #ident {
                fn decode(value: sqlx::postgres::PgValueRef<'r>) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                    let value = <&'r str as sqlx::decode::Decode<'r, sqlx::postgres::Postgres>>::decode(value)?;

                    #values
                }
            }
        ));
    }

    if cfg!(feature = "sqlite") {
        tts.extend(quote!(
            impl<'r> sqlx::decode::Decode<'r, sqlx::sqlite::Sqlite> for #ident {
                fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                    let value = <&'r str as sqlx::decode::Decode<'r, sqlx::sqlite::Sqlite>>::decode(value)?;

                    #values
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_decode_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    check_struct_attributes(input, fields)?;

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "postgres") {
        let ident = &input.ident;

        // extract type generics
        let generics = &input.generics;
        let (_, ty_generics, _) = generics.split_for_impl();

        // add db type for impl generics & where clause
        let mut generics = generics.clone();
        generics.params.insert(0, parse_quote!('r));

        let predicates = &mut generics.make_where_clause().predicates;

        for field in fields {
            let ty = &field.ty;

            predicates.push(parse_quote!(#ty: sqlx::decode::Decode<'r, sqlx::Postgres>));
            predicates.push(parse_quote!(#ty: sqlx::types::Type<sqlx::Postgres>));
        }

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let reads = fields.iter().map(|field| -> Stmt {
            let id = &field.ident;
            let ty = &field.ty;

            parse_quote!(
                let #id = decoder.try_decode::<#ty>()?;
            )
        });

        let names = fields.iter().map(|field| &field.ident);

        tts.extend(quote!(
            impl #impl_generics sqlx::decode::Decode<'r, sqlx::Postgres> for #ident #ty_generics #where_clause {
                fn decode(value: sqlx::postgres::PgValueRef<'r>) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                    let mut decoder = sqlx::postgres::types::PgRecordDecoder::new(value)?;

                    #(#reads)*

                    Ok(#ident {
                        #(#names),*
                    })
                }
            }
        ));
    }

    Ok(tts)
}
