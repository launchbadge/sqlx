use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use crate::collation::Collation;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::io::MySqlBufMutExt;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::types::Type;
use crate::{MySql, MySqlTypeInfo, MySqlValueRef};

impl Type<MySql> for str {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo {
            r#type: ColumnType::VarString, // VARCHAR
            flags: ColumnFlags::empty(),
            // Doesn't matter because we never send this.
            collation: Collation::UTF8MB4_GENERAL_CI,
            max_size: None,
        }
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        // TODO: Support more collations being returned from SQL?
        matches!(
            ty.r#type,
            ColumnType::VarChar
                | ColumnType::Blob
                | ColumnType::TinyBlob
                | ColumnType::MediumBlob
                | ColumnType::LongBlob
                | ColumnType::String
                | ColumnType::VarString
                | ColumnType::Enum
        )
            // Any collation that *isn't* `binary` generally indicates string data.
            // The actual collation used for storage doesn't matter, 
            // because the server will transcode to the charset we specify.
            //
            // See comment in `src/collation.rs` for details.
            && ty.collation != Collation::BINARY
    }
}

impl Encode<'_, MySql> for &'_ str {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        buf.put_str_lenenc(self);

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, MySql> for &'r str {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        value.as_str()
    }
}

impl Type<MySql> for String {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <str as Type<MySql>>::compatible(ty)
    }
}

impl Decode<'_, MySql> for String {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        <&str as Decode<MySql>>::decode(value).map(ToOwned::to_owned)
    }
}

forward_encode_impl!(Arc<str>, &str, MySql);
forward_encode_impl!(Rc<str>, &str, MySql);
forward_encode_impl!(Cow<'_, str>, &str, MySql);
forward_encode_impl!(Box<str>, &str, MySql);
forward_encode_impl!(String, &str, MySql);
