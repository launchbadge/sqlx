mod attributes;
mod decode;
mod encode;
mod has_sql_type;

pub(crate) use decode::expand_derive_decode;
pub(crate) use encode::expand_derive_encode;
pub(crate) use has_sql_type::expand_derive_has_sql_type;

use std::iter::FromIterator;
use syn::DeriveInput;

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
