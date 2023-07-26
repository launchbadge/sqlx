use super::attributes::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_container_attributes, TypeName,
};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, Variant,
};

pub fn expand_derive_type(input: &DeriveInput) -> syn::Result<TokenStream> {
    let attrs = parse_container_attributes(&input.attrs)?;
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
) -> syn::Result<TokenStream> {
    let attr = check_transparent_attributes(input, field)?;

    let ident = &input.ident;
    let ty = &field.ty;

    let generics = &input.generics;
    let (_, ty_generics, _) = generics.split_for_impl();

    if attr.transparent {
        let mut generics = generics.clone();
        let mut array_generics = generics.clone();

        generics
            .params
            .insert(0, parse_quote!(DB: ::sqlx_oldapi::Database));
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::sqlx_oldapi::Type<DB>));
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        array_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::sqlx_oldapi::postgres::PgHasArrayType));
        let (array_impl_generics, _, array_where_clause) = array_generics.split_for_impl();

        return Ok(quote!(
            #[automatically_derived]
            impl #impl_generics ::sqlx_oldapi::Type< DB > for #ident #ty_generics #where_clause {
                fn type_info() -> DB::TypeInfo {
                    <#ty as ::sqlx_oldapi::Type<DB>>::type_info()
                }

                fn compatible(ty: &DB::TypeInfo) -> ::std::primitive::bool {
                    <#ty as ::sqlx_oldapi::Type<DB>>::compatible(ty)
                }
            }
            #[automatically_derived]
            #[cfg(feature = "postgres")]
            impl #array_impl_generics ::sqlx_oldapi::postgres::PgHasArrayType for #ident #ty_generics
            #array_where_clause {
                fn array_type_info() -> ::sqlx_oldapi::postgres::PgTypeInfo {
                    <#ty as ::sqlx_oldapi::postgres::PgHasArrayType>::array_type_info()
                }
            }
        ));
    }

    let mut tts = TokenStream::new();

    if cfg!(feature = "postgres") {
        let ty_name = type_name(ident, attr.type_name.as_ref());

        tts.extend(quote!(
            #[automatically_derived]
            impl ::sqlx_oldapi::Type<::sqlx_oldapi::postgres::Postgres> for #ident #ty_generics {
                fn type_info() -> ::sqlx_oldapi::postgres::PgTypeInfo {
                    ::sqlx_oldapi::postgres::PgTypeInfo::with_name(#ty_name)
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_has_sql_type_weak_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<TokenStream> {
    let attr = check_weak_enum_attributes(input, variants)?;
    let repr = attr.repr.unwrap();
    let ident = &input.ident;
    let ts = quote!(
        #[automatically_derived]
        impl<DB: ::sqlx_oldapi::Database> ::sqlx_oldapi::Type<DB> for #ident
        where
            #repr: ::sqlx_oldapi::Type<DB>,
        {
            fn type_info() -> DB::TypeInfo {
                <#repr as ::sqlx_oldapi::Type<DB>>::type_info()
            }

            fn compatible(ty: &DB::TypeInfo) -> bool {
                <#repr as ::sqlx_oldapi::Type<DB>>::compatible(ty)
            }
        }
    );

    Ok(ts)
}

fn expand_derive_has_sql_type_strong_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<TokenStream> {
    let attributes = check_strong_enum_attributes(input, variants)?;

    let ident = &input.ident;
    let mut tts = TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            #[automatically_derived]
            impl ::sqlx_oldapi::Type<::sqlx_oldapi::MySql> for #ident {
                fn type_info() -> ::sqlx_oldapi::mysql::MySqlTypeInfo {
                    ::sqlx_oldapi::mysql::MySqlTypeInfo::__enum()
                }

                fn compatible(ty: &::sqlx_oldapi::mysql::MySqlTypeInfo) -> ::std::primitive::bool {
                    *ty == ::sqlx_oldapi::mysql::MySqlTypeInfo::__enum()
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        let ty_name = type_name(ident, attributes.type_name.as_ref());

        tts.extend(quote!(
            #[automatically_derived]
            impl ::sqlx_oldapi::Type<::sqlx_oldapi::Postgres> for #ident {
                fn type_info() -> ::sqlx_oldapi::postgres::PgTypeInfo {
                    ::sqlx_oldapi::postgres::PgTypeInfo::with_name(#ty_name)
                }
            }
        ));
    }

    if cfg!(feature = "sqlite") {
        tts.extend(quote!(
            #[automatically_derived]
            impl sqlx::Type<::sqlx_oldapi::Sqlite> for #ident {
                fn type_info() -> ::sqlx_oldapi::sqlite::SqliteTypeInfo {
                    <::std::primitive::str as ::sqlx_oldapi::Type<sqlx::Sqlite>>::type_info()
                }

                fn compatible(ty: &::sqlx_oldapi::sqlite::SqliteTypeInfo) -> ::std::primitive::bool {
                    <&::std::primitive::str as ::sqlx_oldapi::types::Type<sqlx::sqlite::Sqlite>>::compatible(ty)
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_has_sql_type_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let attributes = check_struct_attributes(input, fields)?;

    let ident = &input.ident;
    let mut tts = TokenStream::new();

    if cfg!(feature = "postgres") {
        let ty_name = type_name(ident, attributes.type_name.as_ref());

        tts.extend(quote!(
            #[automatically_derived]
            impl ::sqlx_oldapi::Type<::sqlx_oldapi::Postgres> for #ident {
                fn type_info() -> ::sqlx_oldapi::postgres::PgTypeInfo {
                    ::sqlx_oldapi::postgres::PgTypeInfo::with_name(#ty_name)
                }
            }
        ));
    }

    Ok(tts)
}

fn type_name(ident: &Ident, explicit_name: Option<&TypeName>) -> TokenStream {
    explicit_name.map(|tn| tn.get()).unwrap_or_else(|| {
        let s = ident.to_string();
        quote_spanned!(ident.span()=> #s)
    })
}
