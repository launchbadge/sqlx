use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres},
    types::Type,
};
use byteorder::{BigEndian, ByteOrder};
use geo::{Line, Coordinate};
use std::{mem, num::ParseFloatError};

// <https://www.postgresql.org/docs/12/datatype-geometric.html>

impl Type<Postgres> for Coordinate<f64> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::POINT
    }
}

impl Decode<'_, Postgres> for Coordinate<f64> {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let buf = value.as_bytes()?;

                decode_coordinate_binary(buf)
            }

            PgValueFormat::Text => {
                let s = value.as_str()?;

                let parens: &[_] = &['(', ')'];
                let mut s = s.trim_matches(parens).split(',');

                match (s.next(), s.next()) {
                    (Some(x), Some(y)) => {
                        let x = x
                            .parse()
                            .map_err(|e: ParseFloatError| crate::error::Error::Decode(e.into()))?;
                        let y = y
                            .parse()
                            .map_err(|e: ParseFloatError| crate::error::Error::Decode(e.into()))?;

                        Ok((x, y).into())
                    }

                    _ => Err(Box::new(crate::error::Error::Decode(
                        format!("expecting a value with the format \"(x,y)\"").into(),
                    ))),
                }
            }
        }
    }
}

fn decode_coordinate_binary(buf: &[u8]) -> Result<Coordinate<f64>, BoxDynError> {
    let x = BigEndian::read_f64(buf);

    let y = BigEndian::read_f64(buf);

    Ok((x, y).into())
}

impl Encode<'_, Postgres> for Coordinate<f64> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let _ = Encode::<Postgres>::encode(self.x, buf);
        let _ = Encode::<Postgres>::encode(self.y, buf);

        IsNull::No
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<f64>()
    }
}

impl Type<Postgres> for Line<f64> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::LSEG
    }
}

impl Decode<'_, Postgres> for Line<f64> {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let buf = value.as_bytes()?;
                let start = decode_coordinate_binary(buf)?;
                // buf.advance(Encode::<Postgres>::size_hint(&start));
                let end = decode_coordinate_binary(buf)?;

                Ok(Line::new(start, end))
            }

            // TODO: is there no way to make this make use of the Decode for Coordinate?
            PgValueFormat::Text => {
                let brackets: &[_] = &['[', ']'];
                let mut s = value.as_str()?
                    .trim_matches(brackets)
                    .split(|c| c == '(' || c == ')' || c == ',')
                    .filter_map(|part| if part == "" { None } else { Some(part) });

                match (s.next(), s.next(), s.next(), s.next()) {
                    (Some(x1), Some(y1), Some(x2), Some(y2)) => {
                        let x1 = x1.parse().map_err(|e: ParseFloatError| crate::error::Error::Decode(e.into()))?;
                        let y1 = y1.parse().map_err(|e: ParseFloatError| crate::error::Error::Decode(e.into()))?;
                        let x2 = x2.parse().map_err(|e: ParseFloatError| crate::error::Error::Decode(e.into()))?;
                        let y2 = y2.parse().map_err(|e: ParseFloatError| crate::error::Error::Decode(e.into()))?;

                        let start = Coordinate::from((x1, y1));
                        let end = Coordinate::from((x2, y2));

                        Ok(Line::new(start, end))
                    }

                    _ => Err(Box::new(crate::error::Error::Decode(
                        format!("expecting a value with the format \"[(x,y),(x,y)]\"").into(),
                    ))),
                }
            }
        }
    }
}

impl Encode<'_, Postgres> for Line<f64> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let _ = Encode::<Postgres>::encode_by_ref(&self.start, buf);
        let _ = Encode::<Postgres>::encode_by_ref(&self.end, buf);

        IsNull::No
    }

    fn size_hint(&self) -> usize {
        2 * Encode::<Postgres>::size_hint(&self.start)
    }
}
