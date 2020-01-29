use proc_macro2::Ident;
use quote::quote;
use std::iter::FromIterator;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Arm, Attribute, Block, Data, DataEnum, DataStruct, DeriveInput, Expr, Field,
    Fields, FieldsNamed, FieldsUnnamed, Lit, Meta, MetaNameValue, NestedMeta, Stmt, Variant,
};

macro_rules! assert_attribute {
    ($e:expr, $err:expr, $input:expr) => {
        if !$e {
            return Err(syn::Error::new_spanned($input, $err));
        }
    };
}

struct SqlxAttributes {
    transparent: bool,
    postgres_oid: Option<u32>,
    repr: Option<Ident>,
    rename: Option<String>,
}

fn parse_attributes(input: &[Attribute]) -> syn::Result<SqlxAttributes> {
    let mut transparent = None;
    let mut postgres_oid = None;
    let mut repr = None;
    let mut rename = None;

    macro_rules! fail {
        ($t:expr, $m:expr) => {
            return Err(syn::Error::new_spanned($t, $m));
        };
    }

    macro_rules! try_set {
        ($i:ident, $v:expr, $t:expr) => {
            match $i {
                None => $i = Some($v),
                Some(_) => fail!($t, "duplicate attribute"),
            }
        };
    }

    for attr in input {
        let meta = attr
            .parse_meta()
            .map_err(|e| syn::Error::new_spanned(attr, e))?;
        match meta {
            Meta::List(list) if list.path.is_ident("sqlx") => {
                for value in list.nested.iter() {
                    match value {
                        NestedMeta::Meta(meta) => match meta {
                            Meta::Path(p) if p.is_ident("transparent") => {
                                try_set!(transparent, true, value)
                            }
                            Meta::NameValue(MetaNameValue {
                                path,
                                lit: Lit::Str(val),
                                ..
                            }) if path.is_ident("rename") => try_set!(rename, val.value(), value),
                            Meta::List(list) if list.path.is_ident("postgres") => {
                                for value in list.nested.iter() {
                                    match value {
                                        NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                            path,
                                            lit: Lit::Int(val),
                                            ..
                                        })) if path.is_ident("oid") => {
                                            try_set!(postgres_oid, val.base10_parse()?, value);
                                        }
                                        u => fail!(u, "unexpected value"),
                                    }
                                }
                            }

                            u => fail!(u, "unexpected attribute"),
                        },
                        u => fail!(u, "unexpected attribute"),
                    }
                }
            }
            Meta::List(list) if list.path.is_ident("repr") => {
                if list.nested.len() != 1 {
                    fail!(&list.nested, "expected one value")
                }
                match list.nested.first().unwrap() {
                    NestedMeta::Meta(Meta::Path(p)) if p.get_ident().is_some() => {
                        try_set!(repr, p.get_ident().unwrap().clone(), list);
                    }
                    u => fail!(u, "unexpected value"),
                }
            }
            _ => {}
        }
    }

    Ok(SqlxAttributes {
        transparent: transparent.unwrap_or(false),
        postgres_oid,
        repr,
        rename,
    })
}

fn check_transparent_attributes(input: &DeriveInput, field: &Field) -> syn::Result<()> {
    let attributes = parse_attributes(&input.attrs)?;
    assert_attribute!(
        attributes.transparent,
        "expected #[sqlx(transparent)]",
        input
    );
    #[cfg(feature = "postgres")]
    assert_attribute!(
        attributes.postgres_oid.is_none(),
        "unexpected #[sqlx(postgres(oid = ..))]",
        input
    );
    assert_attribute!(
        attributes.rename.is_none(),
        "unexpected #[sqlx(rename = ..)]",
        field
    );
    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", input);
    let attributes = parse_attributes(&field.attrs)?;
    assert_attribute!(
        !attributes.transparent,
        "unexpected #[sqlx(transparent)]",
        field
    );
    #[cfg(feature = "postgres")]
    assert_attribute!(
        attributes.postgres_oid.is_none(),
        "unexpected #[sqlx(postgres(oid = ..))]",
        field
    );
    assert_attribute!(
        attributes.rename.is_none(),
        "unexpected #[sqlx(rename = ..)]",
        field
    );
    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", field);
    Ok(())
}

fn check_enum_attributes<'a>(
    input: &'a DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<SqlxAttributes> {
    let attributes = parse_attributes(&input.attrs)?;
    assert_attribute!(
        !attributes.transparent,
        "unexpected #[sqlx(transparent)]",
        input
    );
    assert_attribute!(
        attributes.rename.is_none(),
        "unexpected #[sqlx(rename = ..)]",
        input
    );

    for variant in variants {
        let attributes = parse_attributes(&variant.attrs)?;
        assert_attribute!(
            !attributes.transparent,
            "unexpected #[sqlx(transparent)]",
            variant
        );
        #[cfg(feature = "postgres")]
        assert_attribute!(
            attributes.postgres_oid.is_none(),
            "unexpected #[sqlx(postgres(oid = ..))]",
            variant
        );
        assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", variant);
    }

    Ok(attributes)
}

fn check_weak_enum_attributes(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<Ident> {
    let attributes = check_enum_attributes(input, variants)?;
    #[cfg(feature = "postgres")]
    assert_attribute!(
        attributes.postgres_oid.is_none(),
        "unexpected #[sqlx(postgres(oid = ..))]",
        input
    );
    assert_attribute!(attributes.repr.is_some(), "expected #[repr(..)]", input);
    for variant in variants {
        let attributes = parse_attributes(&variant.attrs)?;
        assert_attribute!(
            attributes.rename.is_none(),
            "unexpected #[sqlx(rename = ..)]",
            variant
        );
    }
    Ok(attributes.repr.unwrap())
}

fn check_strong_enum_attributes(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<SqlxAttributes> {
    let attributes = check_enum_attributes(input, variants)?;
    #[cfg(feature = "postgres")]
    assert_attribute!(
        attributes.postgres_oid.is_some(),
        "expected #[sqlx(postgres(oid = ..))]",
        input
    );
    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", input);
    Ok(attributes)
}

fn check_struct_attributes<'a>(
    input: &'a DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<SqlxAttributes> {
    let attributes = parse_attributes(&input.attrs)?;
    assert_attribute!(
        !attributes.transparent,
        "unexpected #[sqlx(transparent)]",
        input
    );
    #[cfg(feature = "postgres")]
    assert_attribute!(
        attributes.postgres_oid.is_some(),
        "expected #[sqlx(postgres(oid = ..))]",
        input
    );
    assert_attribute!(
        attributes.rename.is_none(),
        "unexpected #[sqlx(rename = ..)]",
        input
    );
    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", input);

    for field in fields {
        let attributes = parse_attributes(&field.attrs)?;
        assert_attribute!(
            !attributes.transparent,
            "unexpected #[sqlx(transparent)]",
            field
        );
        #[cfg(feature = "postgres")]
        assert_attribute!(
            attributes.postgres_oid.is_none(),
            "unexpected #[sqlx(postgres(oid = ..))]",
            field
        );
        assert_attribute!(
            attributes.rename.is_none(),
            "unexpected #[sqlx(rename = ..)]",
            field
        );
        assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", field);
    }

    Ok(attributes)
}

pub(crate) fn expand_derive_encode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let args = parse_attributes(&input.attrs)?;

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
        _ => Err(syn::Error::new_spanned(
            input,
            "expected a tuple struct with a single field",
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
    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(DB: sqlx::Database));
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#ty: sqlx::encode::Encode<DB>));
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote!(
        impl #impl_generics sqlx::encode::Encode<DB> for #ident #ty_generics #where_clause {
            fn encode(&self, buf: &mut std::vec::Vec<u8>) {
                sqlx::encode::Encode::encode(&self.0, buf)
            }
            fn encode_nullable(&self, buf: &mut std::vec::Vec<u8>) -> sqlx::encode::IsNull {
                sqlx::encode::Encode::encode_nullable(&self.0, buf)
            }
            fn size_hint(&self) -> usize {
                sqlx::encode::Encode::size_hint(&self.0)
            }
        }
    ))
}

fn expand_derive_encode_weak_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    let repr = check_weak_enum_attributes(input, &variants)?;

    let ident = &input.ident;

    Ok(quote!(
        impl<DB: sqlx::Database> sqlx::encode::Encode<DB> for #ident where #repr: sqlx::encode::Encode<DB> {
            fn encode(&self, buf: &mut std::vec::Vec<u8>) {
                sqlx::encode::Encode::encode(&(*self as #repr), buf)
            }
            fn encode_nullable(&self, buf: &mut std::vec::Vec<u8>) -> sqlx::encode::IsNull {
                sqlx::encode::Encode::encode_nullable(&(*self as #repr), buf)
            }
            fn size_hint(&self) -> usize {
                sqlx::encode::Encode::size_hint(&(*self as #repr))
            }
        }
    ))
}

fn expand_derive_encode_strong_enum(
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
            value_arms.push(quote!(#ident :: #id => #rename,));
        } else {
            let name = id.to_string();
            value_arms.push(quote!(#ident :: #id => #name,));
        }
    }

    Ok(quote!(
        impl<DB: sqlx::Database> sqlx::encode::Encode<DB> for #ident where str: sqlx::encode::Encode<DB> {
            fn encode(&self, buf: &mut std::vec::Vec<u8>) {
                let val = match self {
                    #(#value_arms)*
                };
                <str as sqlx::encode::Encode<DB>>::encode(val, buf)
            }
            fn size_hint(&self) -> usize {
                let val = match self {
                    #(#value_arms)*
                };
                <str as sqlx::encode::Encode<DB>>::size_hint(val)
            }
        }
    ))
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
            predicates.push(parse_quote!(#ty: sqlx::encode::Encode<sqlx::Postgres>));
            predicates.push(parse_quote!(sqlx::Postgres: sqlx::types::HasSqlType<#ty>));
        }
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let mut writes: Vec<Block> = Vec::new();
        for field in fields {
            let id = &field.ident;
            let ty = &field.ty;
            writes.push(parse_quote!({
                // write oid
                let info = <sqlx::Postgres as sqlx::types::HasSqlType<#ty>>::type_info();
                buf.extend(&info.oid().to_be_bytes());

                // write zeros for length
                buf.extend(&[0; 4]);

                let start = buf.len();
                sqlx::encode::Encode::<sqlx::Postgres>::encode(&self. #id, buf);
                let end = buf.len();
                let size = end - start;

                // replaces zeros with actual length
                buf[start-4..start].copy_from_slice(&(size as u32).to_be_bytes());
            }));
        }

        let mut sizes: Vec<Expr> = Vec::new();
        for field in fields {
            let id = &field.ident;
            let ty = &field.ty;
            sizes.push(
                parse_quote!(<#ty as sqlx::encode::Encode<sqlx::Postgres>>::size_hint(&self. #id)),
            );
        }

        tts.extend(quote!(
            impl #impl_generics sqlx::encode::Encode<sqlx::Postgres> for #ident #ty_generics #where_clause {
                fn encode(&self, buf: &mut std::vec::Vec<u8>) {
                    buf.extend(&(#column_count as u32).to_be_bytes());
                    #(#writes)*
                }
                fn size_hint(&self) -> usize {
                    4 + #column_count * (4 + 4) + #(#sizes)+*
                }
            }
        ));
    }

    Ok(tts)
}

pub(crate) fn expand_derive_decode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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

pub(crate) fn expand_derive_has_sql_type(
    input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
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
        _ => Err(syn::Error::new_spanned(
            input,
            "expected a tuple struct with a single field",
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
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    // add db type for clause
    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: sqlx::types::HasSqlType<#ty>));
    let (_, _, where_clause) = generics.split_for_impl();

    let mut tts = proc_macro2::TokenStream::new();

    if cfg!(feature = "mysql") {
        tts.extend(quote!(
            impl #impl_generics sqlx::types::HasSqlType< #ident #ty_generics > for sqlx::MySql #where_clause {
                fn type_info() -> Self::TypeInfo {
                    <Self as HasSqlType<#ty>>::type_info()
                }
            }
        ));
    }

    if cfg!(feature = "postgres") {
        tts.extend(quote!(
            impl #impl_generics sqlx::types::HasSqlType< #ident #ty_generics > for sqlx::Postgres #where_clause {
                fn type_info() -> Self::TypeInfo {
                    <Self as HasSqlType<#ty>>::type_info()
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

pub(crate) fn expand_derive_type(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let encode_tts = expand_derive_encode(input)?;
    let decode_tts = expand_derive_decode(input)?;
    let has_sql_type_tts = expand_derive_has_sql_type(input)?;

    let combined = proc_macro2::TokenStream::from_iter(
        encode_tts
            .into_iter()
            .chain(decode_tts)
            .chain(has_sql_type_tts),
    );
    Ok(combined)
}
