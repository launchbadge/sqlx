use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::types::HasSqlType;
use crate::Postgres;
use std::convert::TryInto;

/// read a struct field and advance the buffer
pub fn decode_struct_field<T: Decode<Postgres>>(buf: &mut &[u8]) -> Result<T, DecodeError>
where
    Postgres: HasSqlType<T>,
{
    if buf.len() < 8 {
        return Err(DecodeError::Message(std::boxed::Box::new(
            "Not enough data sent",
        )));
    }

    let oid = u32::from_be_bytes(std::convert::TryInto::try_into(&buf[0..4]).unwrap());
    if oid != <Postgres as HasSqlType<T>>::type_info().oid() {
        return Err(DecodeError::Message(std::boxed::Box::new("Invalid oid")));
    }

    let len = u32::from_be_bytes(buf[4..8].try_into().unwrap()) as usize;

    if buf.len() < 8 + len {
        return Err(DecodeError::Message(std::boxed::Box::new(
            "Not enough data sent",
        )));
    }

    let raw = &buf[8..8 + len];
    let value = T::decode(raw)?;

    *buf = &buf[8 + len..];

    Ok(value)
}

pub fn encode_struct_field<T: Encode<Postgres>>(buf: &mut Vec<u8>, value: &T)
where
    Postgres: HasSqlType<T>,
{
    // write oid
    let info = <Postgres as HasSqlType<T>>::type_info();
    buf.extend(&info.oid().to_be_bytes());

    // write zeros for length
    buf.extend(&[0; 4]);

    let start = buf.len();
    value.encode(buf);
    let end = buf.len();
    let size = end - start;

    // replaces zeros with actual length
    buf[start - 4..start].copy_from_slice(&(size as u32).to_be_bytes());
}
