mod attributes;
mod decode;
mod encode;
mod row;
mod r#type;

pub(crate) use decode::expand_derive_decode;
pub(crate) use encode::expand_derive_encode;
pub(crate) use r#type::expand_derive_type;
pub(crate) use row::expand_derive_from_row;

use self::attributes::RenameAll;
use heck::{KebabCase, ShoutySnakeCase, SnakeCase};
use std::iter::FromIterator;
use syn::DeriveInput;

pub(crate) fn expand_derive_type_encode_decode(
    input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let encode_tts = expand_derive_encode(input)?;
    let decode_tts = expand_derive_decode(input)?;
    let type_tts = expand_derive_type(input)?;

    let combined = proc_macro2::TokenStream::from_iter(
        encode_tts.into_iter().chain(decode_tts).chain(type_tts),
    );

    Ok(combined)
}

pub(crate) fn rename_all(s: &str, pattern: RenameAll) -> String {
    match pattern {
        RenameAll::LowerCase => s.to_lowercase(),
        RenameAll::SnakeCase => s.to_snake_case(),
        RenameAll::UpperCase => s.to_uppercase(),
        RenameAll::ScreamingSnakeCase => s.to_shouty_snake_case(),
        RenameAll::KebabCase => s.to_kebab_case(),
    }
}
