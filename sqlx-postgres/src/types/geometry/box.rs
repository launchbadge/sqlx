use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::bytes::Buf;
use std::str::FromStr;

const ERROR: &str = "error decoding BOX";

/// ## Postgres Geometric Box type
///
/// Description: Rectangular box
/// Representation: `((x1,y1),(x2,y2))`
///
/// Boxes are represented by pairs of points that are opposite corners of the box. Values of type box are specified using any of the following syntaxes:
///
/// ```text
/// ( ( x1 , y1 ) , ( x2 , y2 ) )
/// ( x1 , y1 ) , ( x2 , y2 )
///   x1 , y1   ,   x2 , y2
/// ```
/// where `(x1,y1) and (x2,y2)` are any two opposite corners of the box.
/// Any two opposite corners can be supplied on input, but the values will be reordered as needed to store the upper right and lower left corners, in that order.
///
/// See https://www.postgresql.org/docs/16/datatype-geometric.html#DATATYPE-GEOMETRIC-BOXES
#[derive(Debug, Clone, PartialEq)]
pub struct PgBox {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Type<Postgres> for PgBox {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("box")
    }
}

impl PgHasArrayType for PgBox {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_box")
    }
}

impl<'r> Decode<'r, Postgres> for PgBox {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.format() {
            PgValueFormat::Text => Ok(PgBox::from_str(value.as_str()?)?),
            PgValueFormat::Binary => Ok(PgBox::from_bytes(value.as_bytes()?)?),
        }
    }
}

impl<'q> Encode<'q, Postgres> for PgBox {
    fn produces(&self) -> Option<PgTypeInfo> {
        Some(PgTypeInfo::with_name("box"))
    }

    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.serialize(buf)?;
        Ok(IsNull::No)
    }
}

impl FromStr for PgBox {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitised = s.replace(['(', ')', '[', ']', ' '], "");
        let mut parts = sanitised.splitn(4, ",");

        let x1 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get x1 from {}", ERROR, s))?;

        let y1 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get y1 from {}", ERROR, s))?;

        let x2 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get x2 from {}", ERROR, s))?;

        let y2 = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get y2 from {}", ERROR, s))?;

        Ok(PgBox { x1, y1, x2, y2 })
    }
}

impl PgBox {
    fn from_bytes(mut bytes: &[u8]) -> Result<PgBox, BoxDynError> {
        let x1 = bytes.get_f64();
        let y1 = bytes.get_f64();
        let x2 = bytes.get_f64();
        let y2 = bytes.get_f64();

        Ok(PgBox { x1, y1, x2, y2 })
    }

    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), String> {
        let min_x = &self.x1.min(self.x2);
        let min_y = &self.y1.min(self.y2);
        let max_x = &self.x1.max(self.x2);
        let max_y = &self.y1.max(self.y2);

        buff.extend_from_slice(&max_x.to_be_bytes());
        buff.extend_from_slice(&max_y.to_be_bytes());
        buff.extend_from_slice(&min_x.to_be_bytes());
        buff.extend_from_slice(&min_y.to_be_bytes());

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
mod box_tests {

    use std::str::FromStr;

    use super::PgBox;

    const BOX_BYTES: &[u8] = &[
        64, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 192, 0, 0, 0, 0, 0, 0, 0, 192, 0, 0, 0,
        0, 0, 0, 0,
    ];

    #[test]
    fn can_deserialise_box_type_bytes_in_order() {
        let pg_box = PgBox::from_bytes(BOX_BYTES).unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x1: 2.,
                y1: 2.,
                x2: -2.,
                y2: -2.
            }
        )
    }

    #[test]
    fn can_deserialise_box_type_str_first_syntax() {
        let pg_box = PgBox::from_str("[( 1, 2), (3, 4 )]").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }
    #[test]
    fn can_deserialise_box_type_str_second_syntax() {
        let pg_box = PgBox::from_str("(( 1, 2), (3, 4 ))").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_box_type_str_third_syntax() {
        let pg_box = PgBox::from_str("(1, 2), (3, 4 )").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_box_type_str_fourth_syntax() {
        let pg_box = PgBox::from_str("1, 2, 3, 4").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x1: 1.,
                y1: 2.,
                x2: 3.,
                y2: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_box_type_str_float() {
        let pg_box = PgBox::from_str("(1.1, 2.2), (3.3, 4.4)").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x1: 1.1,
                y1: 2.2,
                x2: 3.3,
                y2: 4.4
            }
        );
    }

    #[test]
    fn can_serialise_box_type_in_order() {
        let pg_box = PgBox {
            x1: 2.,
            x2: -2.,
            y1: -2.,
            y2: 2.,
        };
        assert_eq!(pg_box.serialize_to_vec(), BOX_BYTES,)
    }

    #[test]
    fn can_serialise_box_type_out_of_order() {
        let pg_box = PgBox {
            x1: -2.,
            x2: 2.,
            y1: 2.,
            y2: -2.,
        };
        assert_eq!(pg_box.serialize_to_vec(), BOX_BYTES,)
    }

    #[test]
    fn can_order_box() {
        let pg_box = PgBox {
            x1: -2.,
            x2: 2.,
            y1: 2.,
            y2: -2.,
        };
        let bytes = pg_box.serialize_to_vec();

        let pg_box = PgBox::from_bytes(&bytes).unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x1: 2.,
                y1: 2.,
                x2: -2.,
                y2: -2.
            }
        )
    }
}
