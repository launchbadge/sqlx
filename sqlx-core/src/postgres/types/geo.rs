use crate::decode::Decode;
use crate::encode::Encode;
use crate::types::Type;
use crate::postgres::protocol::TypeId;
use crate::postgres::{ PgData, PgValue, PgRawBuffer, PgTypeInfo, Postgres };
use crate::io::Buf;
use std::mem;
use geo::Coordinate;
use byteorder::BigEndian;

// <https://www.postgresql.org/docs/12/datatype-geometric.html>

impl Type<Postgres> for Coordinate<f64> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::POINT, "POINT")
    }
}

impl<'de> Decode<'de, Postgres> for Coordinate<f64> {
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(mut buf) => {
                let x = buf.get_f64::<BigEndian>()?;
                println!("we have what is hopefully x: {}", x);
                
                let y = buf.get_f64::<BigEndian>()?;
                println!("is this a y? {}", y);

                Ok((x, y).into())
            }

            PgData::Text(s) => {
                let parens: &[_] = &['(', ')'];
                let mut s = s.trim_matches(parens).split(',');

                match (s.next(), s.next()) {
                    (Some(x), Some(y)) => {
                        let x = x.parse().map_err(crate::Error::decode)?;
                        let y = y.parse().map_err(crate::Error::decode)?;

                        Ok((x, y).into())
                    }

                    _ => Err(crate::Error::Decode(format!("expecting a value with the format \"(x,y)\"").into()))
                }
            }
        }
    }
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
