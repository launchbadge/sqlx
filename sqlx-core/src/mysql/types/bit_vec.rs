use bit_vec::BitVec;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mysql::io::MySqlBufMutExt;
use crate::mysql::protocol::text::ColumnType;
use crate::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
use crate::types::Type;

impl Type<MySql> for BitVec {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Bit)
    }
}

impl Encode<'_, MySql> for BitVec {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.put_bytes_lenenc(&self.to_bytes());

        IsNull::No
    }
}

impl Decode<'_, MySql> for BitVec {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(BitVec::from_bytes(value.as_bytes()?))
    }
}
