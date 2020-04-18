use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::Buf;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgData, PgRawBuffer, PgTypeInfo, PgValue, Postgres};
use crate::types::Type;
use byteorder::BigEndian;
use geo::{Coordinate, Line};
use std::mem;

// <https://www.postgresql.org/docs/12/datatype-geometric.html>

impl Type<Postgres> for Coordinate<f64> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::POINT, "POINT")
    }
}

impl<'de> Decode<'de, Postgres> for Coordinate<f64> {
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(buf) => decode_coordinate_binary(buf),

            PgData::Text(s) => {
                let parens: &[_] = &['(', ')'];
                let mut s = s.trim_matches(parens).split(',');

                match (s.next(), s.next()) {
                    (Some(x), Some(y)) => {
                        let x = x.parse().map_err(crate::Error::decode)?;
                        let y = y.parse().map_err(crate::Error::decode)?;

                        Ok((x, y).into())
                    }

                    _ => Err(crate::Error::Decode(
                        format!("expecting a value with the format \"(x,y)\"").into(),
                    )),
                }
            }
        }
    }
}

fn decode_coordinate_binary(mut buf: &[u8]) -> crate::Result<Coordinate<f64>> {
    let x = buf.get_f64::<BigEndian>()?;

    let y = buf.get_f64::<BigEndian>()?;

    Ok((x, y).into())
}

impl Encode<Postgres> for Coordinate<f64> {
    fn encode(&self, buf: &mut PgRawBuffer) {
        Encode::<Postgres>::encode(&self.x, buf);
        Encode::<Postgres>::encode(&self.y, buf);
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<f64>()
    }
}

impl Type<Postgres> for Line<f64> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::LSEG, "LSEG")
    }
}

impl<'de> Decode<'de, Postgres> for Line<f64> {
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(mut buf) => {
                let start = decode_coordinate_binary(buf)?;
                buf.advance(Encode::<Postgres>::size_hint(&start));
                let end = decode_coordinate_binary(buf)?;

                Ok(Line::new(start, end))
            }

            // TODO: is there no way to make this make use of the Decode for Coordinate?
            PgData::Text(s) => {
                let brackets: &[_] = &['[', ']'];
                let mut s = s
                    .trim_matches(brackets)
                    .split(|c| c == '(' || c == ')' || c == ',')
                    .filter_map(|part| if part == "" { None } else { Some(part) });

                match (s.next(), s.next(), s.next(), s.next()) {
                    (Some(x1), Some(y1), Some(x2), Some(y2)) => {
                        let x1 = x1.parse().map_err(crate::Error::decode)?;
                        let y1 = y1.parse().map_err(crate::Error::decode)?;
                        let x2 = x2.parse().map_err(crate::Error::decode)?;
                        let y2 = y2.parse().map_err(crate::Error::decode)?;

                        let start = Coordinate::from((x1, y1));
                        let end = Coordinate::from((x2, y2));

                        Ok(Line::new(start, end))
                    }

                    _ => Err(crate::Error::Decode(
                        format!("expecting a value with the format \"[(x,y),(x,y)]\"").into(),
                    )),
                }
            }
        }
    }
}

impl Encode<Postgres> for Line<f64> {
    fn encode(&self, buf: &mut PgRawBuffer) {
        Encode::<Postgres>::encode(&self.start, buf);
        Encode::<Postgres>::encode(&self.end, buf);
    }

    fn size_hint(&self) -> usize {
        2 * Encode::<Postgres>::size_hint(&self.start)
    }
}
