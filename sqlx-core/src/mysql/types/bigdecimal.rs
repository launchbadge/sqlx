use bigdecimal::BigDecimal;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::Buf;
use crate::mysql::protocol::TypeId;
use crate::mysql::{MySql, MySqlData, MySqlTypeInfo, MySqlValue};
use crate::types::Type;
use crate::Error;
use std::str::FromStr;

impl Type<MySql> for BigDecimal {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::NEWDECIMAL)
    }
}

impl Encode<MySql> for BigDecimal {
    fn encode(&self, buf: &mut Vec<u8>) {
        let size = Encode::<MySql>::size_hint(self) - 1;
        assert!(size <= std::u8::MAX as usize, "Too large size");
        buf.push(size as u8);
        let s = self.to_string();
        buf.extend_from_slice(s.as_bytes());
    }

    fn size_hint(&self) -> usize {
        let s = self.to_string();
        s.as_bytes().len() + 1
    }
}

impl Decode<'_, MySql> for BigDecimal {
    fn decode(value: MySqlValue) -> crate::Result<Self> {
        match value.try_get()? {
            MySqlData::Binary(mut binary) => {
                let _len = binary.get_u8()?;
                let s = std::str::from_utf8(binary).map_err(Error::decode)?;
                Ok(BigDecimal::from_str(s).map_err(Error::decode)?)
            }
            MySqlData::Text(s) => {
                let s = std::str::from_utf8(s).map_err(Error::decode)?;
                Ok(BigDecimal::from_str(s).map_err(Error::decode)?)
            }
        }
    }
}

#[test]
fn test_encode_decimal() {
    let v: BigDecimal = BigDecimal::from_str("-1.05").unwrap();
    let mut buf: Vec<u8> = vec![];
    <BigDecimal as Encode<MySql>>::encode(&v, &mut buf);
    assert_eq!(buf, vec![0x05, b'-', b'1', b'.', b'0', b'5']);

    let v: BigDecimal = BigDecimal::from_str("-105000").unwrap();
    let mut buf: Vec<u8> = vec![];
    <BigDecimal as Encode<MySql>>::encode(&v, &mut buf);
    assert_eq!(buf, vec![0x07, b'-', b'1', b'0', b'5', b'0', b'0', b'0']);

    let v: BigDecimal = BigDecimal::from_str("0.00105").unwrap();
    let mut buf: Vec<u8> = vec![];
    <BigDecimal as Encode<MySql>>::encode(&v, &mut buf);
    assert_eq!(buf, vec![0x07, b'0', b'.', b'0', b'0', b'1', b'0', b'5']);
}

#[test]
fn test_decode_decimal() {
    let buf: Vec<u8> = vec![0x05, b'-', b'1', b'.', b'0', b'5'];
    let v = <BigDecimal as Decode<'_, MySql>>::decode(MySqlValue::binary(
        MySqlTypeInfo::new(TypeId::NEWDECIMAL),
        buf.as_slice(),
    ))
    .unwrap();
    assert_eq!(v.to_string(), "-1.05");

    let buf: Vec<u8> = vec![0x04, b'0', b'.', b'0', b'5'];
    let v = <BigDecimal as Decode<'_, MySql>>::decode(MySqlValue::binary(
        MySqlTypeInfo::new(TypeId::NEWDECIMAL),
        buf.as_slice(),
    ))
    .unwrap();
    assert_eq!(v.to_string(), "0.05");

    let buf: Vec<u8> = vec![0x06, b'-', b'9', b'0', b'0', b'0', b'0'];
    let v = <BigDecimal as Decode<'_, MySql>>::decode(MySqlValue::binary(
        MySqlTypeInfo::new(TypeId::NEWDECIMAL),
        buf.as_slice(),
    ))
    .unwrap();
    assert_eq!(v.to_string(), "-90000");
}
