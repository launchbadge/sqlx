use std::convert::TryInto;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::{MySql, MySqlValue};
use crate::types::Type;
use crate::Error;
use std::str::from_utf8;

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
impl Type<MySql> for f32 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::FLOAT)
    }
}

impl Encode<MySql> for f32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i32 as Encode<MySql>>::encode(&(self.to_bits() as i32), buf);
    }
}

impl<'de> Decode<'de, MySql> for f32 {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => buf
                .read_i32::<LittleEndian>()
                .map_err(crate::Error::decode)
                .map(|value| f32::from_bits(value as u32)),

            MySqlValue::Text(s) => from_utf8(s)
                .map_err(Error::decode)?
                .parse()
                .map_err(Error::decode),
        }
    }
}

/// The equivalent MySQL type for `f64` is `DOUBLE`.
///
/// Note that `DOUBLE` is a floating-point type and cannot represent some fractional values
/// exactly.
impl Type<MySql> for f64 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::DOUBLE)
    }
}

impl Encode<MySql> for f64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i64 as Encode<MySql>>::encode(&(self.to_bits() as i64), buf);
    }
}

impl<'de> Decode<'de, MySql> for f64 {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => buf
                .read_i64::<LittleEndian>()
                .map_err(crate::Error::decode)
                .map(|value| f64::from_bits(value as u64)),

            MySqlValue::Text(s) => from_utf8(s)
                .map_err(Error::decode)?
                .parse()
                .map_err(Error::decode),
        }
    }
}
