use proc_macro2::Ident;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, DeriveInput, Field, Lit, Meta, MetaNameValue, NestedMeta, Variant};

macro_rules! assert_attribute {
    ($e:expr, $err:expr, $input:expr) => {
        if !$e {
            return Err(syn::Error::new_spanned($input, $err));
        }
    };
}

pub struct SqlxAttributes {
    pub transparent: bool,
    pub postgres_oid: Option<u32>,
    pub repr: Option<Ident>,
    pub rename: Option<String>,
}

pub fn parse_attributes(input: &[Attribute]) -> syn::Result<SqlxAttributes> {
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

pub fn check_transparent_attributes(input: &DeriveInput, field: &Field) -> syn::Result<()> {
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

pub fn check_enum_attributes<'a>(
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

pub fn check_weak_enum_attributes(
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

pub fn check_strong_enum_attributes(
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

pub fn check_struct_attributes<'a>(
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
