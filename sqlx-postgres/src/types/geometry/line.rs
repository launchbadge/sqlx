use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::bytes::Buf;
use sqlx_core::Error;
use std::str::FromStr;

const ERROR: &str = "error decoding LINE";

/// ## Postgres Geometric Line type
///
/// Description: Infinite line
/// Representation: `{A, B, C}`
///
/// Lines are represented by the linear equation Ax + By + C = 0, where A and B are not both zero.
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
            PgValueFormat::Binary => Ok(PgLine::from_bytes(value.as_bytes()?)?),
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
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s
            .trim_matches(|c| c == '{' || c == '}' || c == ' ')
            .splitn(3, ',');

        let a = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get a from {}", ERROR, s))?;

        let b = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get b from {}", ERROR, s))?;

        let c = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get c from {}", ERROR, s))?;

        Ok(PgLine { a, b, c })
    }
}

impl PgLine {
    fn from_bytes(mut bytes: &[u8]) -> Result<PgLine, BoxDynError> {
        let a = bytes.get_f64();
        let b = bytes.get_f64();
        let c = bytes.get_f64();
        Ok(PgLine { a, b, c })
    }

    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), BoxDynError> {
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

#[cfg(test)]
mod line_tests {

    use std::str::FromStr;

    use super::PgLine;

    const LINE_BYTES: &[u8] = &[
        63, 241, 153, 153, 153, 153, 153, 154, 64, 1, 153, 153, 153, 153, 153, 154, 64, 10, 102,
        102, 102, 102, 102, 102,
    ];

    #[test]
    fn can_deserialise_line_type_bytes() {
        let line = PgLine::from_bytes(LINE_BYTES).unwrap();
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
