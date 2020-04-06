use bigdecimal::{BigDecimal, Signed};
use num_bigint::{BigInt, Sign};

use crate::decode::Decode;
use crate::encode::{Encode};
use crate::types::Type;
use crate::mysql::protocol::TypeId;
use crate::mysql::{MySql, MySqlValue, MySqlTypeInfo, MySqlData};
use crate::Error;
use crate::io::Buf;

const SIGN_NEG: u8 = 0x2D;
const SCALE_START: u8 = 0x2E;

impl Type<MySql> for BigDecimal {
  fn type_info() -> MySqlTypeInfo {
    MySqlTypeInfo::new(TypeId::NEWDECIMAL)
  }
}

impl Encode<MySql> for BigDecimal {
  fn encode(&self, buf: &mut Vec<u8>) {
    let size = Encode::<MySql>::size_hint(self) - 1;

    assert!(size <= u8::MAX as usize, "Too large size");

    buf.push(size as u8);

    if self.is_negative() {
      buf.push(SIGN_NEG);
    }

    let (bi, scale) = self.as_bigint_and_exponent();
    let (_, mut radix) =  bi.to_radix_be(10);
    let mut scale_index: Option<usize> = None;

    if scale < 0 {
      radix.append(&mut vec![0u8; -scale as usize]);
    } else {
      let scale = scale as usize;
      if scale >= radix.len() {
        let mut radix_temp = vec![0u8; scale - radix.len() + 1];
        radix_temp.append(&mut radix);
        radix = radix_temp;
        scale_index = Some(0);
      } else {
        scale_index = Some(radix.len() - scale - 1);
      }
    }

    for (i, data) in radix.iter().enumerate() {
      buf.push(*data + 0x30);
      if let Some(si) = scale_index {
        if si == i {
          buf.push(SCALE_START);
          scale_index = None;
        }
      }
    }
  }

  /// 15, -2 => 1500
  /// 15, 1 => 1.5
  /// 15, 2 => 0.15
  /// 15, 3 => 0.015

  fn size_hint(&self) -> usize {
    let (bi, scale) = self.as_bigint_and_exponent();
    let (_, radix) = bi.to_radix_be(10);
    let mut s = radix.len();

    if scale < 0 {
      s = s + (-scale) as usize
    } else if scale > 0 {
      let scale = scale as usize;
      if scale >= s {
        s = scale + 1
      }
      s = s + 1;
    }

    if self.is_negative() {
      s = s + 1;
    }
    s + 1
  }
}

impl Decode<'_, MySql> for BigDecimal {
  fn decode(value: MySqlValue) -> crate::Result<Self> {
    match value.try_get()? {
      MySqlData::Binary(mut binary) => {
        let len = binary.get_u8()?;
        let mut negative = false;
        let mut scale: Option<i64> = None;
        let mut v: Vec<u8> = Vec::with_capacity(len as usize);

        loop {
          if binary.len() < 1 {
            break
          }
          let data = binary.get_u8()?;
          match data {
            SIGN_NEG => {
              if !negative {
                negative = true;
              } else {
                return Err(Error::Decode(format!("Unexpected byte: {:X?}", data).into()));
              }
            },
            SCALE_START => {
              if scale.is_none() {
                scale = Some(0);
              } else {
                return Err(Error::Decode(format!("Unexpected byte: {:X?}", data).into()));
              }
            },
            0x30..=0x39 => {
              scale = scale.map(|s| s + 1);
              v.push(data - 0x30);
            },
            _ => return Err(Error::Decode(format!("Unexpected byte: {:X?}", data).into())),
          }
        }

        let r = BigInt::from_radix_be(
          if negative { Sign::Minus } else { Sign::Plus },
          v.as_slice(),
          10,
        ).ok_or(Error::Decode("Can't convert to BigInt".into()))?;

        Ok(BigDecimal::new(r, scale.unwrap_or(0)))
      },
      MySqlData::Text(_) => {
        Err(Error::Decode(
          "`BigDecimal` can only be decoded from the binary protocol".into(),
        ))
      },
    }
  }
}

#[test]
fn test_encode_decimal() {
  let v = BigDecimal::new(BigInt::from(-105), 2);
  let mut buf: Vec<u8> = vec![];
  v.encode(&mut buf);
  assert_eq!(buf, vec![0x05, 0x2D, 0x31, 0x2E, 0x30, 0x35]);

  let v = BigDecimal::new(BigInt::from(-105), -3);
  let mut buf: Vec<u8> = vec![];
  v.encode(&mut buf);
  assert_eq!(buf, vec![0x07, 0x2D, 0x31, 0x30, 0x35, 0x30, 0x30, 0x30]);

  let v = BigDecimal::new(BigInt::from(105), 5);
  let mut buf: Vec<u8> = vec![];
  v.encode(&mut buf);
  assert_eq!(buf, vec![0x07, 0x30, 0x2E, 0x30, 0x30, 0x31, 0x30, 0x35]);
}

#[test]
fn test_decode_decimal() {
  let buf: Vec<u8> = vec![0x05, 0x2D, 0x31, 0x2E, 0x30, 0x35];
  let v = BigDecimal::decode(MySqlValue::binary(
    MySqlTypeInfo::new(TypeId::NEWDECIMAL), buf.as_slice(),
  )).unwrap();
  assert_eq!(v.to_string(), "-1.05");

  let buf: Vec<u8> = vec![0x04, 0x30, 0x2E, 0x30, 0x35];
  let v = BigDecimal::decode(MySqlValue::binary(
    MySqlTypeInfo::new(TypeId::NEWDECIMAL), buf.as_slice(),
  )).unwrap();
  assert_eq!(v.to_string(), "0.05");

  let buf: Vec<u8> = vec![0x06, 0x2D, 0x39, 0x30, 0x30, 0x30, 0x30];
  let v = BigDecimal::decode(MySqlValue::binary(
    MySqlTypeInfo::new(TypeId::NEWDECIMAL), buf.as_slice(),
  )).unwrap();
  assert_eq!(v.to_string(), "-90000");
}