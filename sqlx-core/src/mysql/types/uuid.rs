use uuid::Uuid;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mysql::io::MySqlBufMutExt;
use crate::mysql::protocol::text::ColumnType;
use crate::mysql::{MySql, MySqlTypeInfo, MySqlValueFormat, MySqlValueRef};
use crate::types::Type;

impl Type<MySql> for Uuid {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Blob)
    }
}

impl Encode<'_, MySql> for Uuid {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.put_bytes_lenenc(self);

        IsNull::No
    }
}

impl Decode<'_, MySql> for Uuid {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            MySqlValueFormat::Binary => Uuid::from_slice(value.as_bytes()?),
            MySqlValueFormat::Text => value.as_str()?.parse(),
        }
        .map_err(Into::into)
    }
}
