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
/// Representation: `((x_upper_right,y_upper_right),(x_lower_left,y_lower_left))`
///
/// Boxes are represented by pairs of points that are opposite corners of the box. Values of type box are specified using any of the following syntaxes:
///
/// ```text
/// ( ( x_upper_right , y_upper_right ) , ( x_lower_left , y_lower_left ) )
/// ( x_upper_right , y_upper_right ) , ( x_lower_left , y_lower_left )
///   x_upper_right , y_upper_right   ,   x_lower_left , y_lower_left
/// ```
/// where `(x_upper_right,y_upper_right) and (x_lower_left,y_lower_left)` are any two opposite corners of the box.
/// Any two opposite corners can be supplied on input, but the values will be reordered as needed to store the upper right and lower left corners, in that order.
///
/// See https://www.postgresql.org/docs/16/datatype-geometric.html#DATATYPE-GEOMETRIC-BOXES
#[derive(Debug, Clone, PartialEq)]
pub struct PgBox {
    pub x_upper_right: f64,
    pub y_upper_right: f64,
    pub x_lower_left: f64,
    pub y_lower_left: f64,
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
        let mut parts = sanitised.split(',');

        let x_upper_right = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get x_upper_right from {}", ERROR, s))?;

        let y_upper_right = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get y_upper_right from {}", ERROR, s))?;

        let x_lower_left = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get x_lower_left from {}", ERROR, s))?;

        let y_lower_left = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get y_lower_left from {}", ERROR, s))?;

        if parts.next().is_some() {
            return Err(format!("{}: too many numbers inputted in {}", ERROR, s).into());
        }

        Ok(PgBox {
            x_upper_right,
            y_upper_right,
            x_lower_left,
            y_lower_left,
        })
    }
}

impl PgBox {
    fn from_bytes(mut bytes: &[u8]) -> Result<PgBox, BoxDynError> {
        let x_upper_right = bytes.get_f64();
        let y_upper_right = bytes.get_f64();
        let x_lower_left = bytes.get_f64();
        let y_lower_left = bytes.get_f64();

        Ok(PgBox {
            x_upper_right,
            y_upper_right,
            x_lower_left,
            y_lower_left,
        })
    }

    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), String> {
        let min_x = &self.x_upper_right.min(self.x_lower_left);
        let min_y = &self.y_upper_right.min(self.y_lower_left);
        let max_x = &self.x_upper_right.max(self.x_lower_left);
        let max_y = &self.y_upper_right.max(self.y_lower_left);

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
                x_upper_right: 2.,
                y_upper_right: 2.,
                x_lower_left: -2.,
                y_lower_left: -2.
            }
        )
    }

    #[test]
    fn can_deserialise_box_type_str_first_syntax() {
        let pg_box = PgBox::from_str("[( 1, 2), (3, 4 )]").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x_upper_right: 1.,
                y_upper_right: 2.,
                x_lower_left: 3.,
                y_lower_left: 4.
            }
        );
    }
    #[test]
    fn can_deserialise_box_type_str_second_syntax() {
        let pg_box = PgBox::from_str("(( 1, 2), (3, 4 ))").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x_upper_right: 1.,
                y_upper_right: 2.,
                x_lower_left: 3.,
                y_lower_left: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_box_type_str_third_syntax() {
        let pg_box = PgBox::from_str("(1, 2), (3, 4 )").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x_upper_right: 1.,
                y_upper_right: 2.,
                x_lower_left: 3.,
                y_lower_left: 4.
            }
        );
    }

    #[test]
    fn can_deserialise_box_type_str_fourth_syntax() {
        let pg_box = PgBox::from_str("1, 2, 3, 4").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x_upper_right: 1.,
                y_upper_right: 2.,
                x_lower_left: 3.,
                y_lower_left: 4.
            }
        );
    }

    #[test]
    fn cannot_deserialise_too_many_numbers() {
        let input_str = "1, 2, 3, 4, 5";
        let pg_box = PgBox::from_str(input_str);
        assert!(pg_box.is_err());
        if let Err(err) = pg_box {
            assert_eq!(
                err.to_string(),
                format!("error decoding BOX: too many numbers inputted in {input_str}")
            )
        }
    }

    #[test]
    fn cannot_deserialise_too_few_numbers() {
        let input_str = "1, 2, 3 ";
        let pg_box = PgBox::from_str(input_str);
        assert!(pg_box.is_err());
        if let Err(err) = pg_box {
            assert_eq!(
                err.to_string(),
                format!("error decoding BOX: could not get y_lower_left from {input_str}")
            )
        }
    }

    #[test]
    fn cannot_deserialise_invalid_numbers() {
        let input_str = "1, 2, 3, FOUR";
        let pg_box = PgBox::from_str(input_str);
        assert!(pg_box.is_err());
        if let Err(err) = pg_box {
            assert_eq!(
                err.to_string(),
                format!("error decoding BOX: could not get y_lower_left from {input_str}")
            )
        }
    }

    #[test]
    fn can_deserialise_box_type_str_float() {
        let pg_box = PgBox::from_str("(1.1, 2.2), (3.3, 4.4)").unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x_upper_right: 1.1,
                y_upper_right: 2.2,
                x_lower_left: 3.3,
                y_lower_left: 4.4
            }
        );
    }

    #[test]
    fn can_serialise_box_type_in_order() {
        let pg_box = PgBox {
            x_upper_right: 2.,
            x_lower_left: -2.,
            y_upper_right: -2.,
            y_lower_left: 2.,
        };
        assert_eq!(pg_box.serialize_to_vec(), BOX_BYTES,)
    }

    #[test]
    fn can_serialise_box_type_out_of_order() {
        let pg_box = PgBox {
            x_upper_right: -2.,
            x_lower_left: 2.,
            y_upper_right: 2.,
            y_lower_left: -2.,
        };
        assert_eq!(pg_box.serialize_to_vec(), BOX_BYTES,)
    }

    #[test]
    fn can_order_box() {
        let pg_box = PgBox {
            x_upper_right: -2.,
            x_lower_left: 2.,
            y_upper_right: 2.,
            y_lower_left: -2.,
        };
        let bytes = pg_box.serialize_to_vec();

        let pg_box = PgBox::from_bytes(&bytes).unwrap();
        assert_eq!(
            pg_box,
            PgBox {
                x_upper_right: 2.,
                y_upper_right: 2.,
                x_lower_left: -2.,
                y_lower_left: -2.
            }
        )
    }
}
