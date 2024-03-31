use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::types::Type;
use crate::{error::BoxDynError, PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, Cursor, Write};
use std::num::ParseIntError;
use std::str;
use std::str::FromStr;

#[derive(Debug)]
pub struct Lexeme {
    word: String,
    positions: Vec<u16>,
}

#[derive(Debug)]
pub struct TsVector {
    words: Vec<Lexeme>,
}

impl Display for TsVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

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

    /// Decode binary data into [`TsVector`] based on the binary data format defined in
    /// https://github.com/postgres/postgres/blob/252dcb32397f64a5e1ceac05b29a271ab19aa960/src/backend/utils/adt/tsvector.c#L399
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(bytes);
        let mut words = vec![];

        let num_lexemes = reader.read_u32::<BigEndian>()?;

        for _ in 0..num_lexemes {
            let mut lexeme = vec![];

            reader.read_until(b'\0', &mut lexeme)?;

            let num_positions = reader.read_u16::<BigEndian>()?;
            let mut positions = Vec::<u16>::with_capacity(num_positions as usize);

            if num_positions > 0 {
                for _ in 0..num_positions {
                    let position = reader.read_u16::<BigEndian>()?;
                    positions.push(position);
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

impl TryInto<Vec<u8>> for &TsVector {
    type Error = BoxDynError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let buf: &mut Vec<u8> = &mut vec![];

        buf.write_u32::<BigEndian>(u32::try_from(self.words.len())?)?;

        for lexeme in &self.words {
            buf.write(lexeme.word.as_bytes())?;
            buf.write(&[b'\0'])?;

            buf.write_u16::<BigEndian>(u16::try_from(lexeme.positions.len())?)?;

            if !lexeme.positions.is_empty() {
                for position in &lexeme.positions {
                    buf.write_u16::<BigEndian>(*position)?;
                }
            }
        }

        buf.flush()?;

        Ok(buf.to_vec())
    }
}

impl FromStr for TsVector {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words: Vec<Lexeme> = vec![];

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
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        if let Ok(encoded_ts_vector) = <&TsVector as TryInto<Vec<u8>>>::try_into(self) {
            buf.extend_from_slice(encoded_ts_vector.as_slice());

            IsNull::No
        } else {
            IsNull::Yes
        }
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
