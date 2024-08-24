use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::{PgPoint, Type};
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::Error;
use std::str::FromStr;

const BYTE_WIDTH: usize = 8;

/// Postgres Geometric Path type
///
/// Storage size: 16+16n bytes
/// Description: Open path or Closed path (similar to polygon)
/// Representation: ((x1,y1),(x2,y2))
///
/// See https://www.postgresql.org/docs/16/datatype-geometric.html#DATATYPE-GEOMETRIC-PATH
#[derive(Debug, Clone, PartialEq)]
pub struct PgPath {
    pub points: Vec<PgPoint>,
}

impl Type<Postgres> for PgPath {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("path")
    }
}

impl PgHasArrayType for PgPath {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_path")
    }
}

impl<'r> Decode<'r, Postgres> for PgPath {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.format() {
            PgValueFormat::Text => Ok(PgPath::from_str(value.as_str()?)?),
            PgValueFormat::Binary => Ok(pg_path_from_bytes(value.as_bytes()?)?),
        }
    }
}

impl<'q> Encode<'q, Postgres> for PgPath {
    fn produces(&self) -> Option<PgTypeInfo> {
        Some(PgTypeInfo::with_name("path"))
    }

    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.serialize(buf)?;
        Ok(IsNull::No)
    }
}

impl FromStr for PgPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitised = s.replace(&['(', ')', '[', ']', ' '][..], "");
        let mut parts = sanitised.splitn(4, ",");

        let mut points = vec![];

        while let (Some(x_str), Some(y_str)) = (parts.next(), parts.next()) {
            let x = parse_float_from_str(x_str, "could not get x")?;
            let y = parse_float_from_str(y_str, "could not get y")?;

            let point = PgPoint { x, y };
            points.push(point);
        }

        if !points.is_empty() {
            return Ok(PgPath { points });
        }

        Err(Error::Decode(
            format!("could not get path from {}", s).into(),
        ))
    }
}

fn pg_path_from_bytes(bytes: &[u8]) -> Result<PgPath, Error> {
    let mut points = vec![];

    let steps = bytes.len() / BYTE_WIDTH;

    for n in (0..steps).step_by(2) {
        let x = get_f64_from_bytes(bytes, BYTE_WIDTH * n)?;
        let y = get_f64_from_bytes(bytes, BYTE_WIDTH * (n + 1))?;
        points.push(PgPoint { x, y })
    }

    Ok(PgPath { points })
}

impl PgPath {
    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), Error> {
        for point in &self.points {
            buff.extend_from_slice(&point.x.to_be_bytes());
            buff.extend_from_slice(&point.y.to_be_bytes());
        }
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
            format!("Could not decode path bytes: {:?}", bytes).into(),
        ))?
        .try_into()
        .map(f64::from_be_bytes)
        .map_err(|err| Error::Decode(format!("Invalid bytes slice: {:?}", err).into()))
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.parse().map_err(|_| Error::Decode(error_msg.into()))
}

#[cfg(test)]
mod path_tests {

    use std::str::FromStr;

    use crate::types::PgPoint;

    use super::{pg_path_from_bytes, PgPath};

    const LINE_SEGMENT_BYTES: &[u8] = &[
        63, 241, 153, 153, 153, 153, 153, 154, 64, 1, 153, 153, 153, 153, 153, 154, 64, 10, 102,
        102, 102, 102, 102, 102, 64, 17, 153, 153, 153, 153, 153, 154,
    ];

    #[test]
    fn can_deserialise_path_type_byes() {
        let path = pg_path_from_bytes(LINE_SEGMENT_BYTES).unwrap();
        assert_eq!(
            path,
            PgPath {
                points: vec![PgPoint { x: 1.1, y: 2.2 }, PgPoint { x: 3.3, y: 4.4 }]
            }
        )
    }

    #[test]
    fn can_deserialise_path_type_str_first_syntax() {
        let path = PgPath::from_str("[( 1, 2), (3, 4 )]").unwrap();
        assert_eq!(
            path,
            PgPath {
                points: vec![PgPoint { x: 1., y: 2. }, PgPoint { x: 3., y: 4. }]
            }
        );
    }
    #[test]
    fn can_deserialise_path_type_str_second_syntax() {
        let path = PgPath::from_str("(( 1, 2), (3, 4 ))").unwrap();
        assert_eq!(
            path,
            PgPath {
                points: vec![PgPoint { x: 1., y: 2. }, PgPoint { x: 3., y: 4. }]
            }
        );
    }

    #[test]
    fn can_deserialise_path_type_str_third_syntax() {
        let path = PgPath::from_str("(1, 2), (3, 4 )").unwrap();
        assert_eq!(
            path,
            PgPath {
                points: vec![PgPoint { x: 1., y: 2. }, PgPoint { x: 3., y: 4. }]
            }
        );
    }

    #[test]
    fn can_deserialise_path_type_str_fourth_syntax() {
        let path = PgPath::from_str("1, 2, 3, 4").unwrap();
        assert_eq!(
            path,
            PgPath {
                points: vec![PgPoint { x: 1., y: 2. }, PgPoint { x: 3., y: 4. }]
            }
        );
    }

    #[test]
    fn can_deserialise_path_type_str_float() {
        let path = PgPath::from_str("(1.1, 2.2), (3.3, 4.4)").unwrap();
        assert_eq!(
            path,
            PgPath {
                points: vec![PgPoint { x: 1.1, y: 2.2 }, PgPoint { x: 3.3, y: 4.4 }]
            }
        );
    }

    #[test]
    fn can_serialise_path_type() {
        let path = PgPath {
            points: vec![PgPoint { x: 1.1, y: 2.2 }, PgPoint { x: 3.3, y: 4.4 }],
        };
        assert_eq!(path.serialize_to_vec(), LINE_SEGMENT_BYTES,)
    }
}
