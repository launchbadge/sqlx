mod attributes;
mod decode;
mod encode;
mod row;
mod r#type;

pub use decode::expand_derive_decode;
pub use encode::expand_derive_encode;
pub use r#type::expand_derive_type;
pub use row::expand_derive_from_row;

use self::attributes::RenameAll;
use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use syn::{DeriveInput, Ident};

pub fn expand_derive_type_encode_decode(
    input: &DeriveInput,
    crate_name: Ident,
) -> syn::Result<TokenStream> {
    let encode_tts = expand_derive_encode(input, &crate_name)?;
    let decode_tts = expand_derive_decode(input, &crate_name)?;
    let type_tts = expand_derive_type(input, &crate_name)?;

    let combined = TokenStream::from_iter(encode_tts.into_iter().chain(decode_tts).chain(type_tts));

    Ok(combined)
}

pub(crate) fn rename_all(s: &str, pattern: RenameAll) -> String {
    match pattern {
        RenameAll::LowerCase => s.to_lowercase(),
        RenameAll::SnakeCase => s.to_snake_case(),
        RenameAll::UpperCase => s.to_uppercase(),
        RenameAll::ScreamingSnakeCase => s.to_shouty_snake_case(),
        RenameAll::KebabCase => s.to_kebab_case(),
        RenameAll::CamelCase => s.to_lower_camel_case(),
        RenameAll::PascalCase => s.to_upper_camel_case(),
    }
}
