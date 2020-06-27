use super::attributes::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_child_attributes, parse_container_attributes,
};
use super::rename_all;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, FieldsNamed,
    FieldsUnnamed, Lifetime, LifetimeDef, Stmt, Variant,
};

pub fn expand_derive_encode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let args = parse_container_attributes(&input.attrs)?;

    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) if unnamed.len() == 1 => {
            expand_derive_encode_transparent(&input, unnamed.first().unwrap())
        }
        Data::Enum(DataEnum { variants, .. }) => match args.repr {
            Some(_) => expand_derive_encode_weak_enum(input, variants),
            None => expand_derive_encode_strong_enum(input, variants),
        },
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => expand_derive_encode_struct(input, named),
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

fn expand_derive_encode_transparent(
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
    let lifetime = Lifetime::new("'q", Span::call_site());
    let mut generics = generics.clone();
    generics
        .params
        .insert(0, LifetimeDef::new(lifetime.clone()).into());

    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#ty: sqlx::encode::Encode<#lifetime, DB>));

    let (impl_generics, _, _) = generics.split_for_impl();

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl #impl_generics sqlx::encode::Encode<#lifetime, sqlx::MySql> for #ident #ty_generics where #ty: sqlx::encode::Encode<#lifetime, sqlx::MySql> {
                fn encode_by_ref(&self, buf: &mut <sqlx::MySql as sqlx::database::HasArguments<#lifetime>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::MySql>>::encode_by_ref(&self.0, buf)
                }

                fn produces(&self) -> Option<sqlx::mysql::MySqlTypeInfo> {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::MySql>>::produces(&self.0)
                }

                fn size_hint(&self) -> usize {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::MySql>>::size_hint(&self.0)
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        tts.extend(quote!(
            impl #impl_generics sqlx::encode::Encode<#lifetime, sqlx::Postgres> for #ident #ty_generics where #ty: sqlx::encode::Encode<#lifetime, sqlx::Postgres> {
                fn encode_by_ref(&self, buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<#lifetime>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::Postgres>>::encode_by_ref(&self.0, buf)
                }

                fn produces(&self) -> Option<sqlx::postgres::PgTypeInfo> {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::Postgres>>::produces(&self.0)
                }

                fn size_hint(&self) -> usize {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::Postgres>>::size_hint(&self.0)
                }
            }
        ));
    }

    if cfg!(feature = "sqlite") {
        tts.extend(quote!(
            impl #impl_generics sqlx::encode::Encode<#lifetime, sqlx::Sqlite> for #ident #ty_generics where #ty: sqlx::encode::Encode<#lifetime, sqlx::Sqlite> {
                fn encode_by_ref(&self, buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<#lifetime>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::Sqlite>>::encode_by_ref(&self.0, buf)
                }

                fn produces(&self) -> Option<sqlx::sqlite::SqliteTypeInfo> {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::Sqlite>>::produces(&self.0)
                }

                fn size_hint(&self) -> usize {
                    <#ty as sqlx::encode::Encode<#lifetime, sqlx::Sqlite>>::size_hint(&self.0)
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_encode_weak_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attr = check_weak_enum_attributes(input, &variants)?;
    let repr = attr.repr.unwrap();

    let ident = &input.ident;

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl<'q> sqlx::encode::Encode<'q, sqlx::MySql> for #ident where #repr: sqlx::encode::Encode<'q, sqlx::MySql> {
                fn encode_by_ref(&self, buf: &mut <sqlx::MySql as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    <#repr as sqlx::encode::Encode<sqlx::MySql>>::encode_by_ref(&(*self as #repr), buf)
                }

                fn produces(&self) -> Option<sqlx::mysql::MySqlTypeInfo> {
                    <#repr as sqlx::encode::Encode<sqlx::MySql>>::produces(&(*self as #repr))
                }

                fn size_hint(&self) -> usize {
                    <#repr as sqlx::encode::Encode<sqlx::MySql>>::size_hint(&(*self as #repr))
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        tts.extend(quote!(
            impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for #ident where #repr: sqlx::encode::Encode<'q, sqlx::Postgres> {
                fn encode_by_ref(&self, buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    <#repr as sqlx::encode::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as #repr), buf)
                }

                fn produces(&self) -> Option<sqlx::postgres::PgTypeInfo> {
                    <#repr as sqlx::encode::Encode<sqlx::Postgres>>::produces(&(*self as #repr))
                }

                fn size_hint(&self) -> usize {
                    <#repr as sqlx::encode::Encode<sqlx::Postgres>>::size_hint(&(*self as #repr))
                }
            }
        ));
    }

    if cfg!(feature = "sqlite") {
        tts.extend(quote!(
            impl<'q> sqlx::encode::Encode<'q, sqlx::Sqlite> for #ident where #repr: sqlx::encode::Encode<'q, sqlx::Sqlite> {
                fn encode_by_ref(&self, buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    <#repr as sqlx::encode::Encode<sqlx::Sqlite>>::encode_by_ref(&(*self as #repr), buf)
                }

                fn produces(&self) -> Option<sqlx::sqlite::SqliteTypeInfo> {
                    <#repr as sqlx::encode::Encode<sqlx::Sqlite>>::produces(&(*self as #repr))
                }

                fn size_hint(&self) -> usize {
                    <#repr as sqlx::encode::Encode<sqlx::Sqlite>>::size_hint(&(*self as #repr))
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_encode_strong_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let cattr = check_strong_enum_attributes(input, &variants)?;

    let ident = &input.ident;

    let mut value_arms = Vec::new();
    for v in variants {
        let id = &v.ident;
        let attributes = parse_child_attributes(&v.attrs)?;

        if let Some(rename) = attributes.rename {
            value_arms.push(quote!(#ident :: #id => #rename,));
        } else if let Some(pattern) = cattr.rename_all {
            let name = rename_all(&*id.to_string(), pattern);

            value_arms.push(quote!(#ident :: #id => #name,));
        } else {
            let name = id.to_string();
            value_arms.push(quote!(#ident :: #id => #name,));
        }
    }

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl<'q> sqlx::encode::Encode<'q, sqlx::MySql> for #ident where &'q str: sqlx::encode::Encode<'q, sqlx::MySql> {
                fn encode_by_ref(&self, buf: &mut <sqlx::MySql as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&str as sqlx::encode::Encode<'q, sqlx::MySql>>::encode(val, buf)
                }

                fn size_hint(&self) -> usize {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&str as sqlx::encode::Encode<'q, sqlx::MySql>>::size_hint(&val)
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        tts.extend(quote!(
            impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for #ident where &'q str: sqlx::encode::Encode<'q, sqlx::Postgres> {
                fn encode_by_ref(&self, buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&str as sqlx::encode::Encode<'q, sqlx::Postgres>>::encode(val, buf)
                }

                fn size_hint(&self) -> usize {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&str as sqlx::encode::Encode<'q, sqlx::Postgres>>::size_hint(&val)
                }
            }
        ));
    }

    if cfg!(feature = "sqlite") {
        tts.extend(quote!(
            impl<'q> sqlx::encode::Encode<'q, sqlx::Sqlite> for #ident where &'q str: sqlx::encode::Encode<'q, sqlx::Sqlite> {
                fn encode_by_ref(&self, buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&str as sqlx::encode::Encode<'q, sqlx::Sqlite>>::encode(val, buf)
                }

                fn size_hint(&self) -> usize {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&str as sqlx::encode::Encode<'q, sqlx::Sqlite>>::size_hint(&val)
                }
            }
        ));
    }

    Ok(tts)
}

fn expand_derive_encode_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    check_struct_attributes(input, &fields)?;

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

            predicates.push(parse_quote!(#ty: for<'q> sqlx::encode::Encode<'q, sqlx::Postgres>));
            predicates.push(parse_quote!(#ty: sqlx::types::Type<sqlx::Postgres>));
        }

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let writes = fields.iter().map(|field| -> Stmt {
            let id = &field.ident;

            parse_quote!(
                encoder.encode(&self. #id);
            )
        });

        let sizes = fields.iter().map(|field| -> Expr {
            let id = &field.ident;
            let ty = &field.ty;

            parse_quote!(
                <#ty as sqlx::encode::Encode<sqlx::Postgres>>::size_hint(&self. #id)
            )
        });

        tts.extend(quote!(
            impl #impl_generics sqlx::encode::Encode<'_, sqlx::Postgres> for #ident #ty_generics #where_clause {
                fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
                    let mut encoder = sqlx::postgres::types::PgRecordEncoder::new(buf);

                    #(#writes)*

                    encoder.finish();

                    sqlx::encode::IsNull::No
                }

                fn size_hint(&self) -> usize {
                    #column_count * (4 + 4) // oid (int) and length (int) for each column
                        + #(#sizes)+* // sum of the size hints for each column
                }
            }
        ));
    }

    Ok(tts)
}
