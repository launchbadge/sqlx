use serde::{Deserialize, Serialize};

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::{Json, Type};
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

impl<T> Type<Mssql> for Json<T> {
    fn type_info() -> MssqlTypeInfo {
        // SQL Server has no native JSON type; JSON is stored as NVARCHAR
        MssqlTypeInfo::new("NVARCHAR")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <&str as Type<Mssql>>::compatible(ty)
    }
}

impl<T> Encode<'_, Mssql> for Json<T>
where
    T: Serialize,
{
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        let json_string = self.encode_to_string()?;
        buf.push(MssqlArgumentValue::String(json_string));
        Ok(IsNull::No)
    }
}

impl<'r, T> Decode<'r, Mssql> for Json<T>
where
    T: Deserialize<'r> + 'r,
{
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        Json::decode_from_string(value.as_str()?)
    }
}
