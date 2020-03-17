use super::attributes::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_attributes,
};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, Variant,
};

pub fn expand_derive_type(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = parse_attributes(&input.attrs)?;
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) if unnamed.len() == 1 => {
            expand_derive_has_sql_type_transparent(input, unnamed.first().unwrap())
        }
        Data::Enum(DataEnum { variants, .. }) => match attrs.repr {
            Some(_) => expand_derive_has_sql_type_weak_enum(input, variants),
            None => expand_derive_has_sql_type_strong_enum(input, variants),
        },
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => expand_derive_has_sql_type_struct(input, named),
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

fn expand_derive_has_sql_type_transparent(
    input: &DeriveInput,
    field: &Field,
) -> syn::Result<proc_macro2::TokenStream> {
    check_transparent_attributes(input, field)?;

    let ident = &input.ident;
    let ty = &field.ty;

    // extract type generics
    let generics = &input.generics;
    let (_, ty_generics, _) = generics.split_for_impl();

    // add db type for clause
    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(DB: sqlx::Database));
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#ty: sqlx::types::Type<DB>));
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let mut tts = proc_macro2::TokenStream::new();

    // if cfg!(feature = "mysql") {
    tts.extend(quote!(
        impl #impl_generics sqlx::types::Type< DB > for #ident #ty_generics #where_clause {
            fn type_info() -> DB::TypeInfo {
                <#ty as sqlx::Type<DB>>::type_info()
            }
        }
    ));

    // }

    // if cfg!(feature = "postgres") {
    //     tts.extend(quote!(
    //         impl #impl_generics sqlx::types::HasSqlType< sqlx::Postgres > #ident #ty_generics #where_clause {
    //             fn type_info() -> Self::TypeInfo {
    //                 <Self as HasSqlType<#ty>>::type_info()
    //             }
    //         }
    //     ));
    // }

    Ok(tts)
}

fn expand_derive_has_sql_type_weak_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let repr = check_weak_enum_attributes(input, variants)?;

    let ident = &input.ident;
    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl sqlx::types::HasSqlType< #ident > for sqlx::MySql where Self: sqlx::types::HasSqlType< #repr > {
                fn type_info() -> Self::TypeInfo {
                    <Self as HasSqlType<#repr>>::type_info()
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        tts.extend(quote!(
            impl sqlx::types::HasSqlType< #ident > for sqlx::Postgres where Self: sqlx::types::HasSqlType< #repr > {
                fn type_info() -> Self::TypeInfo {
                    <Self as HasSqlType<#repr>>::type_info()
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_has_sql_type_strong_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attributes = check_strong_enum_attributes(input, variants)?;

    let ident = &input.ident;
    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl sqlx::types::HasSqlType< #ident > for sqlx::MySql {
                fn type_info() -> Self::TypeInfo {
                    sqlx::mysql::MySqlTypeInfo::r#enum()
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        let oid = attributes.postgres_oid.unwrap();
        tts.extend(quote!(
            impl sqlx::types::HasSqlType< #ident > for sqlx::Postgres {
                fn type_info() -> Self::TypeInfo {
                    sqlx::postgres::PgTypeInfo::with_oid(#oid)
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_has_sql_type_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attributes = check_struct_attributes(input, fields)?;

    let ident = &input.ident;
    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "postgres") {
        let oid = attributes.postgres_oid.unwrap();
        tts.extend(quote!(
            impl sqlx::types::HasSqlType< #ident > for sqlx::Postgres {
                fn type_info() -> Self::TypeInfo {
                    sqlx::postgres::PgTypeInfo::with_oid(#oid)
                }
            }
        ));
    }

    Ok(tts)
}
