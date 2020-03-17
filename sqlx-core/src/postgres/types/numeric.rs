use byteorder::BigEndian;

use std::convert::TryInto;

use crate::database::{Database, HasRawValue};
use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgTypeInfo, PgValue, Postgres};
use crate::types::Type;
use crate::Error;

/// Wire representation of a Postgres NUMERIC type
#[derive(Debug, PartialEq, Eq)]
pub struct PgNumeric {
    pub sign: PgNumericSign,
    pub scale: i16,
    pub weight: i16,
    pub digits: Vec<i16>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i16)]
pub enum PgNumericSign {
    Positive = 0x0000,
    Negative = 0x4000,
    NotANumber = -0x4000, // 0xC000
}

impl PgNumericSign {
    fn from_u16(sign: i16) -> crate::Result<PgNumericSign> {
        // https://github.com/postgres/postgres/blob/bcd1c3630095e48bc3b1eb0fc8e8c8a7c851eba1/src/backend/utils/adt/numeric.c#L167-L170
        match sign {
            0x0000 => Ok(PgNumericSign::Positive),
            0x4000 => Ok(PgNumericSign::Negative),
            -0x4000 => Ok(PgNumericSign::NotANumber),
            _ => Err(Error::Decode(
                format!("unknown value for PgNumericSign: {:#04X}", sign).into(),
            )),
        }
    }
}

impl Type<Postgres> for PgNumeric {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::new(TypeId::NUMERIC, "NUMERIC")
    }
}

impl PgNumeric {
    pub(crate) fn from_bytes(mut bytes: &[u8]) -> crate::Result<Self> {
        // https://github.com/postgres/postgres/blob/bcd1c3630095e48bc3b1eb0fc8e8c8a7c851eba1/src/backend/utils/adt/numeric.c#L874
        let num_digits = bytes.get_u16::<BigEndian>()?;
        let weight = bytes.get_i16::<BigEndian>()?;
        let sign = bytes.get_i16::<BigEndian>()?;
        let scale = bytes.get_i16::<BigEndian>()?;

        let digits: Vec<_> = (0..num_digits)
            .map(|_| bytes.get_i16::<BigEndian>())
            .collect::<Result<_, _>>()?;

        Ok(PgNumeric {
            sign: PgNumericSign::from_u16(sign)?,
            scale,
            weight,
            digits,
        })
    }
}

/// ### Note
/// Receiving `PgNumeric` is only supported for the Postgres binary (prepared statements) protocol.
impl Decode<'_, Postgres> for PgNumeric {
    fn decode(value: Option<PgValue>) -> crate::Result<Self> {
        if let PgValue::Binary(bytes) = value.try_into()? {
            Self::from_bytes(bytes)
        } else {
            Err(Error::Decode(
                format!("`PgNumeric` can only be decoded from the binary protocol").into(),
            ))
        }
    }
}

/// ### Panics
///
/// * If `self.digits.len()` overflows `i16`
/// * If any element in `self.digits` is greater than or equal to 10000
impl Encode<Postgres> for PgNumeric {
    fn encode(&self, buf: &mut Vec<u8>) {
        let digits_len: i16 = self
            .digits
            .len()
            .try_into()
            .expect("PgNumeric.digits.len() should not overflow i16");

        buf.put_i16::<BigEndian>(digits_len);
        buf.put_i16::<BigEndian>(self.weight);
        buf.put_i16::<BigEndian>(self.sign as i16);
        buf.put_i16::<BigEndian>(self.scale);

        for &digit in &self.digits {
            assert!(digit < 10000, "PgNumeric digits must be in base-10000");
            buf.put_i16::<BigEndian>(digit);
        }
    }

    fn size_hint(&self) -> usize {
        // 4 i16's plus digits
        8 + self.digits.len() * 2
    }
}
