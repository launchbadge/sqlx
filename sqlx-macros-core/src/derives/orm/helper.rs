use heck::ToLowerCamelCase;
use quote::{__private::TokenStream, quote, ToTokens};
use syn::{DeriveInput, Expr, Field};
use syn::{LitStr, Type};

/// skip field
pub(crate) fn is_transient(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "sqlx", "skip")
}

/// readonly field
pub(crate) fn is_readonly(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "sqlx", "readonly")
}

/// primary key field
pub(crate) fn is_pk(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "sqlx", "pk")
}

/// by field
pub(crate) fn is_by(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "sqlx", "by")
}

/// created_at field
pub(crate) fn is_created_at(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "sqlx", "created_at")
}

/// updated_at field
pub(crate) fn is_updated_at(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "sqlx", "updated_at")
}

/// new_method
pub(crate) fn get_new_method(field: &Field) -> TokenStream {
    match get_attribute_by_key(&field.attrs, "sqlx", "new") {
        None => {
            let class_token = field.ty.to_token_stream();
            quote! {
                #class_token::new()
            }
        }
        Some(method_name) => {
            let method_name: Expr =
                syn::parse_str(&method_name).expect("Failed to parse new method name");
            quote! {
                #method_name
            }
        }
    }
}

/// default check
pub(crate) fn get_is_default_method(field: &Field) -> TokenStream {
    let instance_field = field.ident.as_ref().unwrap();
    match get_attribute_by_key(&field.attrs, "sqlx", "is_default") {
        None => {
            let class_token = field.ty.to_token_stream();
            quote! {
                #instance_field == #class_token::default()
            }
        }
        Some(method_name) => {
            let method_name: Expr =
                syn::parse_str(&method_name).expect("Failed to parse is_default method name");
            quote! {
                #instance_field.#method_name
            }
        }
    }
}

/// table_name
pub(crate) fn get_table_name(input: &DeriveInput) -> String {
    let table_name = get_attribute_by_key(&input.attrs, "sqlx", "rename");
    match table_name {
        None => {
            let table_name = input.ident.to_string().to_lower_camel_case();
            pluralizer::pluralize(table_name.as_str(), 2, false)
        }
        Some(table_name) => table_name,
    }
}

/// field_name if rename
pub(crate) fn get_field_name(field: &Field) -> String {
    get_attribute_by_key(&field.attrs, "sqlx", "rename")
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string().to_lower_camel_case())
}

// make string "?, ?, ?" or "$1, $2, $3"
pub(crate) fn create_insert_placeholders(fields: &[&Field]) -> String {
    let max = fields.len();
    let itr = 1..max + 1;
    itr.into_iter()
        .map(db_placeholder)
        .collect::<Vec<String>>()
        .join(",")
}

pub(crate) fn create_update_placeholders(fields: &[&Field]) -> String {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| format!("{} = {}", get_field_name(f), db_placeholder(i + 1)))
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn db_pool_token() -> (TokenStream, TokenStream) {
    let pool = quote!(sqlx::Pool<sqlx::Postgres>);
    let query_result = quote!(sqlx::postgres::PgQueryResult);
    (pool, query_result)
}

pub(crate) fn db_placeholder(index: usize) -> String {
    format!("${}", index)
}

/// `#[name(value)]` attribute value exist or not
pub(crate) fn has_attribute_value(attrs: &[syn::Attribute], name: &str, value: &str) -> bool {
    for attr in attrs.iter() {
        if !attr.path().is_ident(name) {
            continue;
        }

        let f = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(value) {
                return Ok(());
            }
            Err(meta.error("attribute value not found"))
        });
        if f.is_ok() {
            return true;
        }
    }
    false
}

/// `#[name(key="val")]` Get the value of the name attribute by key
pub(crate) fn get_attribute_by_key(
    attrs: &[syn::Attribute],
    name: &str,
    key: &str,
) -> Option<String> {
    let mut val: Option<String> = None;
    for attr in attrs.iter() {
        if !attr.path().is_ident(name) {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(key) {
                let value = meta.value()?; // this parses the `=`
                let v: LitStr = value.parse()?; // this parses `"val"`
                val = Some(v.value());
                return Ok(());
            }
            Err(meta.error("attribute value not found"))
        })
        .ok();
    }
    val
}

/// whether `Option<inner_type>` returns (whether Option, inner_type).
pub(crate) fn get_option_type(ty: &Type) -> (bool, &Type) {
    get_inner_type(ty, "Option")
}

/// whether inner_type,such as: Option<String>,Vec<String>
/// returns (whether, inner_type).
pub(crate) fn get_inner_type<'a>(ty: &'a Type, name: &str) -> (bool, &'a Type) {
    if let syn::Type::Path(ref path) = ty {
        if let Some(segment) = path.path.segments.first() {
            if segment.ident == name {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = &segment.arguments
                {
                    if let Some(syn::GenericArgument::Type(ty)) = args.first() {
                        return (true, ty);
                    }
                }
            }
        }
    }
    (false, ty)
}
