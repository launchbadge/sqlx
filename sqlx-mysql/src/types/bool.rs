use bytes::BufMut;
use sqlx_core::{decode, encode};
use sqlx_core::{Decode, Encode, Runtime};

use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId, MySqlTypeInfo};

// In MySQL, a boolean is an alias for `TINYINT(1) UNSIGNED`
// the functions below delegate functionality to the `u8` impls

// TODO: accepts(ty) -> ty.is_integer()
// TODO: compatible(ty) -> ty.is_integer()

impl<Rt: Runtime> Encode<MySql, Rt> for bool {
    fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        <u8 as Encode<MySql, Rt>>::encode(&(*self as u8), ty, out)
    }
}

impl<'r, Rt: Runtime> Decode<'r, MySql, Rt> for bool {
    fn decode(raw: MySqlRawValue<'r>) -> decode::Result<Self> {
        Ok(raw.decode::<u8, Rt>()? != 0)
    }
}
