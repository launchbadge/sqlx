use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Json;
use crate::types::Type;
use serde::{Deserialize, Serialize};

impl<T> Type<Mssql> for Json<T> {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::BigVarBinary, 0))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::VarBinary
                | DataType::Binary
                | DataType::BigVarBinary
                | DataType::BigBinary
                | DataType::VarChar
                | DataType::Char
                | DataType::BigVarChar
                | DataType::BigChar
        )
    }
}

impl<'q, T> Encode<'q, Mssql> for Json<T>
where
    T: Serialize,
{
    fn produces(&self) -> Option<MssqlTypeInfo> {
        let size = 0xFF_FF;
        return Some(MssqlTypeInfo(TypeInfo::new(DataType::BigVarBinary, size)));
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        serde_json::to_writer(buf, self).unwrap();
        IsNull::No
    }
}

impl<'r, T: 'r> Decode<'r, Mssql> for Json<T>
where
    T: Deserialize<'r>,
{
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let buf = value.as_bytes()?;
        serde_json::from_slice(buf).map(Json).map_err(Into::into)
    }
}
