use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::types::Type;
use crate::{
    database::HasArguments, error::BoxDynError, PgHasArrayType, PgTypeInfo, PgValueFormat,
    PgValueRef, Postgres,
};
use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::{Display, Formatter, Write};
use std::io::{BufRead, Cursor};
use std::num::ParseIntError;
use std::str;
use std::str::FromStr;

#[derive(Debug)]
pub struct Lexeme {
    word: String,
    positions: Vec<i32>,
}

#[derive(Debug)]
pub struct TsVector {
    words: Vec<Lexeme>,
}

impl Display for TsVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut words = self.words.iter().peekable();

        while let Some(word) = words.next() {
            f.write_str(&format!(
                "'{}':{}",
                word.word,
                word.positions
                    .iter()
                    .map(|pos| pos.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ))?;
            if words.peek().is_some() {
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for TsVector {
    type Error = BoxDynError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(bytes);
        let mut words = vec![];

        let num_lexemes = reader.read_u32::<BigEndian>()?;

        for _ in 0..num_lexemes {
            let mut lexeme = vec![];

            reader.read_until(b'\0', &mut lexeme)?;

            let num_positions = reader.read_u16::<BigEndian>()?;
            let mut positions = Vec::<i32>::with_capacity(num_positions as usize);

            if num_positions > 0 {
                for _ in 0..num_positions {
                    let position = reader.read_u16::<BigEndian>()?;
                    positions.push(position as i32);
                }
            }

            words.push(Lexeme {
                word: str::from_utf8(&lexeme)?.trim_end_matches('\0').to_string(),
                positions,
            });
        }

        Ok(Self { words })
    }
}

impl FromStr for TsVector {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words: Vec<Lexeme> = vec![];

        println!("{s}");

        for word in s.split(' ') {
            if let Some((word, positions)) = word.rsplit_once(':') {
                words.push(Lexeme {
                    word: word
                        .trim_start_matches('\'')
                        .trim_end_matches('\'')
                        .to_string(),
                    positions: positions
                        .split(',')
                        .map(|value| value.parse())
                        .collect::<Result<Vec<_>, _>>()?,
                });
            }
        }

        Ok(TsVector { words })
    }
}

impl Type<Postgres> for TsVector {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TS_VECTOR
    }
}

impl PgHasArrayType for TsVector {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TS_VECTOR_ARRAY
    }
}

impl Encode<'_, Postgres> for TsVector {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'_>>::ArgumentBuffer) -> IsNull {
        buf.extend_from_slice(self.to_string().as_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for TsVector {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let bytes = value.as_bytes()?;
                let ts_vector = bytes.try_into()?;

                Ok(ts_vector)
            }
            PgValueFormat::Text => Ok(value.as_str()?.parse::<TsVector>()?),
        }
    }
}
