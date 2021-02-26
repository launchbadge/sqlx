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

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use sqlx_core::Result;

    use crate::{MySqlRawValue, MySqlRawValueFormat, MySqlTypeInfo};

    #[test]
    fn decode_bool_from_tinyint() -> Result<()> {
        let bytes = Bytes::from_static(b"\x01");
        let val: bool = MySqlRawValue::binary(&bytes, &MySqlTypeInfo::TINYINT_1).decode()?;

        assert_eq!(val, true);

        Ok(())
    }

    #[test]
    fn decode_bool_from_bigint() -> Result<()> {
        let bytes = Bytes::from_static(b"\x01\x00\x00\x00\x00\x00\x00\x00");
        let val: bool = MySqlRawValue::binary(&bytes, &MySqlTypeInfo::BIGINT).decode()?;

        assert_eq!(val, true);

        Ok(())
    }
}
