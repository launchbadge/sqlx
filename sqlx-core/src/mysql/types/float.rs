use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::MySql;
use crate::types::HasSqlType;

/// The equivalent MySQL type for `f32` is `FLOAT`.
///
/// ### Note
/// While we added support for `f32` as `FLOAT` for completeness, we don't recommend using
/// it for any real-life applications as it cannot precisely represent some fractional values,
/// and may be implicitly widened to `DOUBLE` in some cases, resulting in a slightly different
/// value:
///
/// ```rust
/// // Widening changes the equivalent decimal value, these two expressions are not equal
/// // (This is expected behavior for floating points and happens both in Rust and in MySQL)
/// assert_ne!(10.2f32 as f64, 10.2f64);
/// ```
impl HasSqlType<f32> for MySql {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::FLOAT)
    }
}

impl Encode<MySql> for f32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i32 as Encode<MySql>>::encode(&(self.to_bits() as i32), buf);
    }
}

impl Decode<MySql> for f32 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(f32::from_bits(<i32 as Decode<MySql>>::decode(buf)? as u32))
    }
}

/// The equivalent MySQL type for `f64` is `DOUBLE`.
///
/// Note that `DOUBLE` is a floating-point type and cannot represent some fractional values
/// exactly.
impl HasSqlType<f64> for MySql {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::DOUBLE)
    }
}

impl Encode<MySql> for f64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i64 as Encode<MySql>>::encode(&(self.to_bits() as i64), buf);
    }
}

impl Decode<MySql> for f64 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(f64::from_bits(<i64 as Decode<MySql>>::decode(buf)? as u64))
    }
}
