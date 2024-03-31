use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::io::{BufRead, Cursor, Write};

#[derive(Clone, Debug)]
pub struct TsQuery {
    entries: Vec<Entry>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Operators {
    Not = 1,
    And = 2,
    Or = 3,
    Phrase = 4,
}

impl Into<u8> for Operators {
    fn into(self) -> u8 {
        match self {
            Self::Not => 1,
            Self::And => 2,
            Self::Or => 3,
            Self::Phrase => 4,
        }
    }
}

impl TryFrom<u8> for Operators {
    type Error = BoxDynError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Operators::Not),
            2 => Ok(Operators::And),
            3 => Ok(Operators::Or),
            4 => Ok(Operators::Phrase),
            _ => Err(BoxDynError::from("Invalid operator")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Operator {
    operator: Operators,
    distance: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct Value {
    weight: u8,
    text: String,
    prefix: u8,
}

#[derive(Clone, Debug)]
pub enum Entry {
    Operator(Operator),
    Value(Value),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EntryType {
    Value = 1,
    Operator = 2,
}

impl Into<u8> for EntryType {
    fn into(self) -> u8 {
        match self {
            EntryType::Value => 1,
            EntryType::Operator => 2,
        }
    }
}

impl TryFrom<u8> for EntryType {
    type Error = BoxDynError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(EntryType::Value),
            2 => Ok(EntryType::Operator),
            _ => Err(BoxDynError::from("Invalid type")),
        }
    }
}

impl TryFrom<&[u8]> for TsQuery {
    type Error = BoxDynError;

    /// Decode binary data into [`TsQuery`] based on the binary data format defined in
    /// https://github.com/postgres/postgres/blob/252dcb32397f64a5e1ceac05b29a271ab19aa960/src/backend/utils/adt/tsquery.c#L1174
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(bytes);

        let count = reader.read_u32::<BigEndian>()?;

        let mut entries = Vec::<Entry>::with_capacity(count as usize);

        for _ in 0..count {
            let entry_type: EntryType = reader.read_u8()?.try_into()?;

            match entry_type {
                EntryType::Value => {
                    let weight = reader.read_u8()?;
                    let mut text = String::new().into_bytes();

                    reader.read_until(b'\0', &mut text)?;

                    let text = String::from_utf8(text)?;
                    let prefix = reader.read_u8()?;

                    entries.push(Entry::Value(Value {
                        weight,
                        text,
                        prefix,
                    }));
                }
                EntryType::Operator => {
                    let operator: Operators = reader.read_u8()?.try_into()?;
                    let distance = if let Operators::Phrase = operator {
                        Some(reader.read_u16::<BigEndian>()?)
                    } else {
                        None
                    };

                    entries.push(Entry::Operator(Operator { operator, distance }));
                }
            }
        }

        Ok(TsQuery { entries })
    }
}

impl TryInto<Vec<u8>> for &TsQuery {
    type Error = BoxDynError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let buf: &mut Vec<u8> = &mut vec![];

        buf.write_u32::<BigEndian>(u32::try_from(self.entries.len())?)?;

        for entry in &self.entries {
            match entry {
                Entry::Operator(operator) => {
                    buf.write_u8(EntryType::Operator.into())?;
                    buf.write_u8(operator.operator.into())?;

                    if let Some(distance) = operator.distance {
                        buf.write_u16::<BigEndian>(distance)?;
                    }
                }
                Entry::Value(value) => {
                    buf.write_u8(EntryType::Value.into())?;
                    buf.write_u8(value.weight)?;

                    buf.write(value.text.as_bytes())?;
                    buf.write(&[b'\0'])?;

                    buf.write_u8(value.prefix)?;
                }
            }
        }

        buf.flush()?;

        Ok(buf.to_vec())
    }
}

impl Type<Postgres> for TsQuery {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TS_QUERY
    }
}

impl PgHasArrayType for TsQuery {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TS_QUERY_ARRAY
    }
}

impl Encode<'_, Postgres> for TsQuery {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        if let Ok(encoded_ts_query) = <&TsQuery as TryInto<Vec<u8>>>::try_into(self) {
            buf.extend_from_slice(encoded_ts_query.as_slice());

            IsNull::No
        } else {
            IsNull::Yes
        }
    }
}

impl Decode<'_, Postgres> for TsQuery {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let bytes = value.as_bytes()?;
                let ts_query = bytes.try_into()?;

                Ok(ts_query)
            }
            _ => unimplemented!(),
        }
    }
}
