use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::bytes::Buf;
use sqlx_core::Error;
use std::str::FromStr;

const ERROR: &str = "error decoding LSEG";

/// ## Postgres Geometric Line Segment type
///
/// Description: Finite line segment
/// Representation: `((x1,y1),(x2,y2))`
///
///
/// Line segments are represented by pairs of points that are the endpoints of the segment. Values of type lseg are specified using any of the following syntaxes:
/// ```text
/// [ ( x1 , y1 ) , ( x2 , y2 ) ]
/// ( ( x1 , y1 ) , ( x2 , y2 ) )
///   ( x1 , y1 ) , ( x2 , y2 )
///     x1 , y1   ,   x2 , y2
/// ```
/// where `(x1,y1) and (x2,y2)` are the end points of the line segment.
///
/// See https://www.postgresql.org/docs/16/datatype-geometric.html#DATATYPE-LSEG
#[derive(Debug, Clone, PartialEq)]
pub struct PgLSeg {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Type<Postgres> for PgLSeg {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("lseg")
    }
}

impl PgHasArrayType for PgLSeg {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_lseg")
    }
}

impl<'r> Decode<'r, Postgres> for PgLSeg {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.format() {
            PgValueFormat::Text => Ok(PgLSeg::from_str(value.as_str()?)?),
            PgValueFormat::Binary => Ok(PgLSeg::from_bytes(value.as_bytes()?)?),
        }
    }
}

impl<'q> Encode<'q, Postgres> for PgLSeg {
    fn produces(&self) -> Option<PgTypeInfo> {
        Some(PgTypeInfo::with_name("lseg"))
    }

    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.serialize(buf)?;
        Ok(IsNull::No)
    }
}

impl FromStr for PgLSeg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitised = s.replace(['(', ')', '[', ']', ' '], "");
        let mut parts = sanitised.splitn(4, ",");

        let x1 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or(Error::Decode(
                format!("{}: could not get x1 from {}", ERROR, s).into(),
            ))?;

        let y1 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or(Error::Decode(
                format!("{}: could not get y1 from {}", ERROR, s).into(),
            ))?;

        let x2 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or(Error::Decode(
                format!("{}: could not get x2 from {}", ERROR, s).into(),
            ))?;

        let y2 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or(Error::Decode(
                format!("{}: could not get y2 from {}", ERROR, s).into(),
            ))?;

        Ok(PgLSeg { x1, y1, x2, y2 })
    }
}

impl PgLSeg {
    fn from_bytes(mut bytes: &[u8]) -> Result<PgLSeg, BoxDynError> {
        let x1 = bytes.get_f64();
        let y1 = bytes.get_f64();
        let x2 = bytes.get_f64();
        let y2 = bytes.get_f64();

        Ok(PgLSeg { x1, y1, x2, y2 })
    }

    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), BoxDynError> {
        buff.extend_from_slice(&self.x1.to_be_bytes());
        buff.extend_from_slice(&self.y1.to_be_bytes());
        buff.extend_from_slice(&self.x2.to_be_bytes());
        buff.extend_from_slice(&self.y2.to_be_bytes());
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
mod lseg_tests {

    use std::str::FromStr;

    use super::PgLSeg;

    const LINE_SEGMENT_BYTES: &[u8] = &[
        63, 241, 153, 153, 153, 153, 153, 154, 64, 1, 153, 153, 153, 153, 153, 154, 64, 10, 102,
        102, 102, 102, 102, 102, 64, 17, 153, 153, 153, 153, 153, 154,
    ];

    #[test]
    fn can_deserialise_lseg_type_bytes() {
        let lseg = PgLSeg::from_bytes(LINE_SEGMENT_BYTES).unwrap();
        assert_eq!(
            lseg,
            PgLSeg {
                x1: 1.1,
                y1: 2.2,
                x2: 3.3,
                y2: 4.4
            }
        )
    }

    #[test]
    fn can_deserialise_lseg_type_str_first_syntax() {
        let lseg = PgLSeg::from_str("[( 1, 2), (3, 4 )]").unwrap();
        assert_eq!(
            lseg,
            PgLSeg {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }
    #[test]
    fn can_deserialise_lseg_type_str_second_syntax() {
        let lseg = PgLSeg::from_str("(( 1, 2), (3, 4 ))").unwrap();
        assert_eq!(
            lseg,
            PgLSeg {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_lseg_type_str_third_syntax() {
        let lseg = PgLSeg::from_str("(1, 2), (3, 4 )").unwrap();
        assert_eq!(
            lseg,
            PgLSeg {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_lseg_type_str_fourth_syntax() {
        let lseg = PgLSeg::from_str("1, 2, 3, 4").unwrap();
        assert_eq!(
            lseg,
            PgLSeg {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_lseg_type_str_float() {
        let lseg = PgLSeg::from_str("(1.1, 2.2), (3.3, 4.4)").unwrap();
        assert_eq!(
            lseg,
            PgLSeg {
                x1: 1.1,
                y1: 2.2,
                x2: 3.3,
                y2: 4.4
            }
        );
    }

    #[test]
    fn can_serialise_lseg_type() {
        let lseg = PgLSeg {
            x1: 1.1,
            y1: 2.2,
            x2: 3.3,
            y2: 4.4,
        };
        assert_eq!(lseg.serialize_to_vec(), LINE_SEGMENT_BYTES,)
    }
}
