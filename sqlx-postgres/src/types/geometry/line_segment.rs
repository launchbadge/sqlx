use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::Error;
use std::str::FromStr;

const BYTE_WIDTH: usize = 8;

/// Postgres Geometric Line Segment type
///
/// Storage size: 32 bytes
/// Description: Finite line segment
/// Representation: ((x1,y1),(x2,y2))
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
            PgValueFormat::Binary => Ok(pg_lseg_from_bytes(value.as_bytes()?)?),
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
        let sanitised = s.replace(&['(', ')', '[', ']', ' '][..], "");
        let mut parts = sanitised.splitn(4, ",");

        if let (Some(x1_str), Some(y1_str), Some(x2_str), Some(y2_str)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            let x1 = parse_float_from_str(x1_str, "could not get x1")?;
            let y1 = parse_float_from_str(y1_str, "could not get y1")?;
            let x2 = parse_float_from_str(x2_str, "could not get x2")?;
            let y2 = parse_float_from_str(y2_str, "could not get y2")?;

            return Ok(PgLSeg { x1, y1, x2, y2 });
        }

        Err(Error::Decode(
            format!("could not get x1, y1, x2, y2 from {}", s).into(),
        ))
    }
}

fn pg_lseg_from_bytes(bytes: &[u8]) -> Result<PgLSeg, Error> {
    let x1 = get_f64_from_bytes(bytes, 0)?;
    let y1 = get_f64_from_bytes(bytes, BYTE_WIDTH)?;
    let x2 = get_f64_from_bytes(bytes, BYTE_WIDTH * 2)?;
    let y2 = get_f64_from_bytes(bytes, BYTE_WIDTH * 3)?;

    Ok(PgLSeg { x1, y1, x2, y2 })
}

impl PgLSeg {
    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), Error> {
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

fn get_f64_from_bytes(bytes: &[u8], start: usize) -> Result<f64, Error> {
    bytes
        .get(start..start + BYTE_WIDTH)
        .ok_or(Error::Decode(
            format!("Could not decode lseg bytes: {:?}", bytes).into(),
        ))?
        .try_into()
        .map(f64::from_be_bytes)
        .map_err(|err| Error::Decode(format!("Invalid bytes slice: {:?}", err).into()))
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.parse().map_err(|_| Error::Decode(error_msg.into()))
}

#[cfg(test)]
mod lseg_tests {

    use std::str::FromStr;

    use super::{pg_lseg_from_bytes, PgLSeg};

    const LINE_SEGMENT_BYTES: &[u8] = &[
        63, 241, 153, 153, 153, 153, 153, 154, 64, 1, 153, 153, 153, 153, 153, 154, 64, 10, 102,
        102, 102, 102, 102, 102, 64, 17, 153, 153, 153, 153, 153, 154,
    ];

    #[test]
    fn can_deserialise_lseg_type_bytes() {
        let lseg = pg_lseg_from_bytes(LINE_SEGMENT_BYTES).unwrap();
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
