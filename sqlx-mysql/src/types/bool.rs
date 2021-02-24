use sqlx_core::{decode, encode, Type};
use sqlx_core::{Decode, Encode};

use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId, MySqlTypeInfo};

// In MySQL, a boolean is an alias for `TINYINT(1) UNSIGNED`
// the functions below delegate functionality to the `u8` impls

impl Type<MySql> for bool {
    fn type_id() -> MySqlTypeId {
        <u8 as Type<MySql>>::type_id()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <u8 as Type<MySql>>::compatible(ty)
    }
}

impl Encode<MySql> for bool {
    fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        <u8 as Encode<MySql>>::encode(&(*self as u8), ty, out)
    }
}

impl<'r> Decode<'r, MySql> for bool {
    fn decode(raw: MySqlRawValue<'r>) -> decode::Result<Self> {
        Ok(raw.decode::<u8>()? != 0)
    }
}
