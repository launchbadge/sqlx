use std::convert::TryInto;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::row::PgValue;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::Type;

impl Type<Postgres> for bool {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::BOOL, "BOOL")
    }
}

impl Type<Postgres> for [bool] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_BOOL, "BOOL[]")
    }
}
impl Type<Postgres> for Vec<bool> {
    fn type_info() -> PgTypeInfo {
        <[bool] as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }
}

impl<'de> Decode<'de, Postgres> for bool {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(buf) => Ok(buf.get(0).map(|&b| b != 0).unwrap_or_default()),

            PgValue::Text("t") => Ok(true),
            PgValue::Text("f") => Ok(false),

            PgValue::Text(s) => {
                return Err(crate::Error::Decode(
                    format!("unexpected value {:?} for boolean", s).into(),
                ));
            }
        }
    }
}
