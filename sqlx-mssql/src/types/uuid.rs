use uuid::Uuid;

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

impl Type<Mssql> for Uuid {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("UNIQUEIDENTIFIER")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "UNIQUEIDENTIFIER"
    }
}

impl Encode<'_, Mssql> for Uuid {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::Uuid(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for Uuid {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::Uuid(v) => Ok(*v),
            MssqlData::String(ref s) => Ok(Uuid::parse_str(s)?),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected UNIQUEIDENTIFIER, got {:?}", value.data).into()),
        }
    }
}

impl Type<Mssql> for uuid::fmt::Hyphenated {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("UNIQUEIDENTIFIER")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "UNIQUEIDENTIFIER"
    }
}

impl Encode<'_, Mssql> for uuid::fmt::Hyphenated {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::Uuid(*self.as_uuid()));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for uuid::fmt::Hyphenated {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let uuid = Uuid::decode(value)?;
        Ok(uuid.hyphenated())
    }
}
