use super::attributes::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_container_attributes,
};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, Variant,
};

pub fn expand_derive_type(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
) -> syn::Result<proc_macro2::TokenStream> {
    let attr = check_transparent_attributes(input, field)?;

    let ident = &input.ident;
    let ty = &field.ty;

    let generics = &input.generics;
    let (_, ty_generics, _) = generics.split_for_impl();

    if attr.transparent {
        let mut generics = generics.clone();

        let mut tts = proc_macro2::TokenStream::new();

        if cfg!(feature = "mysql") {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: sqlx::Type<sqlx::MySql>));

            let (impl_generics, _, where_clause) = generics.split_for_impl();

            tts.extend(quote!(
                impl #impl_generics sqlx::Type<sqlx::MySql> for #ident #ty_generics #where_clause
                {
                    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
                        <#ty as sqlx::Type<sqlx::MySql>>::type_info()
                    }
                }
            ));
        }

        if cfg!(feature = "postgres") {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: sqlx::Type<sqlx::Postgres>));

            let (impl_generics, _, where_clause) = generics.split_for_impl();

            tts.extend(quote!(
                impl #impl_generics sqlx::Type<sqlx::Postgres> for #ident #ty_generics #where_clause
                {
                    fn type_info() -> sqlx::postgres::PgTypeInfo {
                        <#ty as sqlx::Type<sqlx::Postgres>>::type_info()
                    }
                }
            ));
        }

        if cfg!(feature = "sqlite") {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: sqlx::Type<sqlx::Sqlite>));

            let (impl_generics, _, where_clause) = generics.split_for_impl();

            tts.extend(quote!(
                impl #impl_generics sqlx::Type<sqlx::Sqlite> for #ident #ty_generics #where_clause
                {
                    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
                        <#ty as sqlx::Type<sqlx::Sqlite>>::type_info()
                    }
                }
            ));
        }

        return Ok(tts);
    }

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "postgres") {
        let ty_name = attr.rename.unwrap_or_else(|| ident.to_string());

        tts.extend(quote!(
            impl sqlx::Type< sqlx::postgres::Postgres > for #ident #ty_generics {
                fn type_info() -> sqlx::postgres::PgTypeInfo {
                    sqlx::postgres::PgTypeInfo::with_name(#ty_name)
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_has_sql_type_weak_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attr = check_weak_enum_attributes(input, variants)?;
    let repr = attr.repr.unwrap();
    let ident = &input.ident;

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl sqlx::Type<sqlx::MySql> for #ident
            where
                #repr: sqlx::Type<sqlx::MySql>,
            {
                fn type_info() -> sqlx::mysql::MySqlTypeInfo {
                    <#repr as sqlx::Type<sqlx::MySql>>::type_info()
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        tts.extend(quote!(
            impl sqlx::Type<sqlx::Postgres> for #ident
            where
                #repr: sqlx::Type<sqlx::Postgres>,
            {
                fn type_info() -> sqlx::postgres::PgTypeInfo {
                    <#repr as sqlx::Type<sqlx::Postgres>>::type_info()
                }
            }
        ));
    }

    if cfg!(feature = "sqlite") {
        tts.extend(quote!(
            impl sqlx::Type<sqlx::Sqlite> for #ident
            where
                #repr: sqlx::Type<sqlx::Sqlite>,
            {
                fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
                    <#repr as sqlx::Type<sqlx::Sqlite>>::type_info()
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
            impl sqlx::Type< sqlx::MySql > for #ident {
                fn type_info() -> sqlx::mysql::MySqlTypeInfo {
                    sqlx::mysql::MySqlTypeInfo::__enum()
                }

                fn compatible(ty: &sqlx::mysql::MySqlTypeInfo) -> bool {
                    ty == sqlx::mysql::MySqlTypeInfo::__enum()
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        let ty_name = attributes.rename.unwrap_or_else(|| ident.to_string());

        tts.extend(quote!(
            impl sqlx::Type< sqlx::Postgres > for #ident {
                fn type_info() -> sqlx::postgres::PgTypeInfo {
                    sqlx::postgres::PgTypeInfo::with_name(#ty_name)
                }
            }
        ));
    }

    if cfg!(feature = "sqlite") {
        tts.extend(quote!(
            impl sqlx::Type< sqlx::Sqlite > for #ident {
                fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
                    <str as sqlx::Type<sqlx::Sqlite>>::type_info()
                }

                fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
                    <&str as sqlx::types::Type<sqlx::sqlite::Sqlite>>::compatible(ty)
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
        let ty_name = attributes.rename.unwrap_or_else(|| ident.to_string());

        tts.extend(quote!(
            impl sqlx::Type< sqlx::Postgres > for #ident {
                fn type_info() -> sqlx::postgres::PgTypeInfo {
                    sqlx::postgres::PgTypeInfo::with_name(#ty_name)
                }
            }
        ));
    }

    Ok(tts)
}
