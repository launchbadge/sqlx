use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::Error;
use std::str::FromStr;

const BYTE_WIDTH: usize = 8;

/// Postgres Geometric Point type
///
/// Storage size: 16 bytes
/// Description: Point on a plane
/// Representation: (x, y)
///
/// See https://www.postgresql.org/docs/16/datatype-geometric.html#DATATYPE-GEOMETRIC-POINTS
#[derive(Debug, Clone, PartialEq)]
pub struct PgPoint {
    pub x: f64,
    pub y: f64,
}

impl Type<Postgres> for PgPoint {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("point")
    }
}

impl PgHasArrayType for PgPoint {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_point")
    }
}

impl<'r> Decode<'r, Postgres> for PgPoint {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.format() {
            PgValueFormat::Text => Ok(PgPoint::from_str(value.as_str()?)?),
            PgValueFormat::Binary => Ok(pg_point_from_bytes(value.as_bytes()?)?),
        }
    }
}

impl<'q> Encode<'q, Postgres> for PgPoint {
    fn produces(&self) -> Option<PgTypeInfo> {
        Some(PgTypeInfo::with_name("point"))
    }

    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.serialize(buf)?;
        Ok(IsNull::No)
    }
}

impl FromStr for PgPoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x_str, y_str) = s
            .trim_matches(|c| c == '(' || c == ')' || c == ' ')
            .split_once(',')
            .ok_or(Error::Decode(
                format!("could not get x and y from {}", s).into(),
            ))?;

        let x = parse_float_from_str(x_str, "could not get x")?;
        let y = parse_float_from_str(y_str, "could not get x")?;

        Ok(PgPoint { x, y })
    }
}

fn pg_point_from_bytes(bytes: &[u8]) -> Result<PgPoint, Error> {
    let x = get_f64_from_bytes(bytes, 0)?;
    let y = get_f64_from_bytes(bytes, 8)?;
    Ok(PgPoint { x, y })
}

impl PgPoint {
    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), Error> {
        buff.extend_from_slice(&self.x.to_be_bytes());
        buff.extend_from_slice(&self.y.to_be_bytes());
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
            format!("Could not decode point bytes: {:?}", bytes).into(),
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
mod point_tests {

    use std::str::FromStr;

    use super::{pg_point_from_bytes, PgPoint};

    const POINT_BYTES: &[u8] = &[
        64, 0, 204, 204, 204, 204, 204, 205, 64, 20, 204, 204, 204, 204, 204, 205,
    ];

    #[test]
    fn can_deserialise_point_type_bytes() {
        let point = pg_point_from_bytes(POINT_BYTES).unwrap();
        assert_eq!(point, PgPoint { x: 2.1, y: 5.2 })
    }

    #[test]
    fn can_deserialise_point_type_str() {
        let point = PgPoint::from_str("(2, 3)").unwrap();
        assert_eq!(point, PgPoint { x: 2., y: 3. });
    }

    #[test]
    fn can_deserialise_point_type_str_float() {
        let point = PgPoint::from_str("(2.5, 3.4)").unwrap();
        assert_eq!(point, PgPoint { x: 2.5, y: 3.4 });
    }

    #[test]
    fn can_serialise_point_type() {
        let point = PgPoint { x: 2.1, y: 5.2 };
        assert_eq!(point.serialize_to_vec(), POINT_BYTES,)
    }
}
