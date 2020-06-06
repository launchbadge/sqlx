use byteorder::{ByteOrder, LittleEndian};

use crate::database::{Database, HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::io::MsSqlBufMutExt;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{MsSql, MsSqlTypeInfo, MsSqlValueRef};
use crate::types::Type;

impl Type<MsSql> for str {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::NVarChar, 0))
    }
}

impl Encode<'_, MsSql> for &'_ str {
    fn produces(&self) -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::NVarChar, (self.len() * 2) as u32))
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.put_utf16_str(self);

        IsNull::No
    }
}
