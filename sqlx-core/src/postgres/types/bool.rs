use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgData, PgTypeInfo, PgValue, Postgres};
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
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(buf) => Ok(buf.get(0).map(|&b| b != 0).unwrap_or_default()),

            PgData::Text("t") => Ok(true),
            PgData::Text("f") => Ok(false),

            PgData::Text(s) => Err(decode_err!("unexpected value {:?} for boolean", s)),
        }
    }
}
