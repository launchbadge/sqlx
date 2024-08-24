use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::Error;
use std::str::FromStr;

const BYTE_WIDTH: usize = 8;

/// Postgres Geometric Line type
///
/// Storage size: 24 bytes
/// Description: Infinite line
/// Representation: {A, B, C}
///
/// See https://www.postgresql.org/docs/16/datatype-geometric.html#DATATYPE-LINE
#[derive(Debug, Clone, PartialEq)]
pub struct PgLine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Type<Postgres> for PgLine {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("line")
    }
}

impl PgHasArrayType for PgLine {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_line")
    }
}

impl<'r> Decode<'r, Postgres> for PgLine {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.format() {
            PgValueFormat::Text => Ok(PgLine::from_str(value.as_str()?)?),
            PgValueFormat::Binary => Ok(pg_line_from_bytes(value.as_bytes()?)?),
        }
    }
}

impl<'q> Encode<'q, Postgres> for PgLine {
    fn produces(&self) -> Option<PgTypeInfo> {
        Some(PgTypeInfo::with_name("line"))
    }

    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.serialize(buf)?;
        Ok(IsNull::No)
    }
}

impl FromStr for PgLine {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s
            .trim_matches(|c| c == '{' || c == '}' || c == ' ')
            .splitn(3, ',');

        if let (Some(a_str), Some(b_str), Some(c_str)) = (parts.next(), parts.next(), parts.next())
        {
            let a = parse_float_from_str(a_str, "could not get A")?;
            let b = parse_float_from_str(b_str, "could not get B")?;
            let c = parse_float_from_str(c_str, "could not get C")?;

            return Ok(PgLine { a, b, c });
        }

        Err(Error::Decode(
            format!("could not get A,B,C from {}", s).into(),
        ))
    }
}

fn pg_line_from_bytes(bytes: &[u8]) -> Result<PgLine, Error> {
    let a = get_f64_from_bytes(bytes, 0)?;
    let b = get_f64_from_bytes(bytes, BYTE_WIDTH)?;
    let c = get_f64_from_bytes(bytes, BYTE_WIDTH * 2)?;
    Ok(PgLine { a, b, c })
}

impl PgLine {
    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), Error> {
        buff.extend_from_slice(&self.a.to_be_bytes());
        buff.extend_from_slice(&self.b.to_be_bytes());
        buff.extend_from_slice(&self.c.to_be_bytes());
        Ok(())
    }

    #[cfg(test)]
    fn serialize_to_vec(&self) -> Vec<u8> {
        let mut buff = PgArgumentBuffer::default();
        self.serialize(&mut buff).unwrap();
        buff.to_vec()
    }
}

fn get_f64_from_bytes(bytes: &[u8], start: usize) -> Result<f64, Error> {
    bytes
        .get(start..start + BYTE_WIDTH)
        .ok_or(Error::Decode(
            format!("Could not decode line bytes: {:?}", bytes).into(),
        ))?
        .try_into()
        .map(f64::from_be_bytes)
        .map_err(|err| Error::Decode(format!("Invalid bytes slice: {:?}", err).into()))
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.trim()
        .parse()
        .map_err(|_| Error::Decode(error_msg.into()))
}

#[cfg(test)]
mod line_tests {

    use std::str::FromStr;

    use super::{pg_line_from_bytes, PgLine};

    const LINE_BYTES: &[u8] = &[
        63, 241, 153, 153, 153, 153, 153, 154, 64, 1, 153, 153, 153, 153, 153, 154, 64, 10, 102,
        102, 102, 102, 102, 102,
    ];

    #[test]
    fn can_deserialise_line_type_bytes() {
        let line = pg_line_from_bytes(LINE_BYTES).unwrap();
        assert_eq!(
            line,
            PgLine {
                a: 1.1,
                b: 2.2,
                c: 3.3
            }
        )
    }

    #[test]
    fn can_deserialise_line_type_str() {
        let line = PgLine::from_str("{ 1, 2, 3 }").unwrap();
        assert_eq!(
            line,
            PgLine {
                a: 1.0,
                b: 2.0,
                c: 3.0
            }
        );
    }

    #[test]
    fn can_deserialise_line_type_str_float() {
        let line = PgLine::from_str("{1.1, 2.2, 3.3}").unwrap();
        assert_eq!(
            line,
            PgLine {
                a: 1.1,
                b: 2.2,
                c: 3.3
            }
        );
    }

    #[test]
    fn can_serialise_line_type() {
        let line = PgLine {
            a: 1.1,
            b: 2.2,
            c: 3.3,
        };
        assert_eq!(line.serialize_to_vec(), LINE_BYTES,)
    }
}
