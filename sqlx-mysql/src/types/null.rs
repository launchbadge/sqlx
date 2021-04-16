use sqlx_core::database::{HasOutput, HasRawValue};
use sqlx_core::{decode, encode, Database, Decode, Encode, Null, Type};

use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId, MySqlTypeInfo};

impl Type<MySql> for Null {
    fn type_id() -> MySqlTypeId
    where
        Self: Sized,
    {
        MySqlTypeId::NULL
    }
}

impl Encode<MySql> for Null {
    fn encode(&self, _: &MySqlTypeInfo, _: &mut MySqlOutput<'_>) -> encode::Result {
        Ok(encode::IsNull::Yes)
    }
}

impl<'r> Decode<'r, MySql> for Null {
    fn decode(_: MySqlRawValue<'r>) -> decode::Result<Self> {
        Ok(Self)
    }
}
