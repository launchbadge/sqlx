use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::types::Type;
use crate::{
    error::BoxDynError, PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef,
    Postgres,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use core::fmt;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, Cursor, Write};
use std::num::{IntErrorKind, ParseIntError};
use std::str;
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub struct LexemeMeta {
    position: u16,
    weight: u16,
}

impl From<u16> for LexemeMeta {
    fn from(value: u16) -> Self {
        let weight = (value >> 14) & 0b11;
        let position = value & 0x3fff;

        Self { weight, position }
    }
}

impl From<&LexemeMeta> for u16 {
    fn from(LexemeMeta { weight, position }: &LexemeMeta) -> Self {
        let mut lexeme_meta = 0u16;
        lexeme_meta = (weight << 14) | (position & 0x3fff);
        lexeme_meta = (position & 0xc00) | (weight & 0x3fff);

        lexeme_meta
    }
}

#[derive(Debug)]
pub struct ParseLexemeMetaError {
    kind: IntErrorKind,
}

impl From<ParseIntError> for ParseLexemeMetaError {
    fn from(value: ParseIntError) -> Self {
        Self {
            kind: value.kind().clone(),
        }
    }
}

#[allow(deprecated)]
impl Display for ParseLexemeMetaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for ParseLexemeMetaError {
    fn description(&self) -> &str {
        match self.kind {
            IntErrorKind::Empty => "cannot parse integer from empty string",
            IntErrorKind::InvalidDigit => "invalid digit found in string",
            IntErrorKind::PosOverflow => "number too large to fit in target type",
            IntErrorKind::NegOverflow => "number too small to fit in target type",
            IntErrorKind::Zero => "number would be zero for non-zero type",
            _ => "unknown",
        }
    }
}

impl FromStr for LexemeMeta {
    type Err = ParseLexemeMetaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with(&['A', 'B', 'C', 'D']) {
            let weight_char = s.chars().last().ok_or(ParseLexemeMetaError {
                kind: IntErrorKind::Empty,
            })?;
            let weight = match weight_char {
                'A' => 3,
                'B' => 2,
                'C' => 1,
                'D' => 0,
                _ => {
                    return Err(ParseLexemeMetaError {
                        kind: IntErrorKind::InvalidDigit,
                    })
                }
            };

            let position = s.strip_suffix(weight_char).unwrap_or(s).parse::<u16>()?;

            Ok(Self { weight, position })
        } else {
            Ok(Self {
                weight: 0,
                position: s.parse()?,
            })
        }
    }
}

#[derive(Debug)]
pub struct Lexeme {
    word: String,
    positions: Vec<LexemeMeta>,
}

impl Lexeme {
    pub fn word(&self) -> &str {
        self.word.as_str()
    }
}

#[derive(Debug)]
pub struct TsVector {
    words: Vec<Lexeme>,
}

impl TsVector {
    pub fn words(&self) -> &Vec<Lexeme> {
        &self.words
    }
}

impl Display for TsVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let mut words = self.words.iter().peekable();

        while let Some(Lexeme { positions, word }) = words.next() {
            // Add escaping for any single quotes within the word.
            let word = word.replace("'", "''");

            if positions.is_empty() {
                f.write_str(&format!("'{}'", word))?;
            } else {
                let position = positions
                    .into_iter()
                    .map(|LexemeMeta { position, weight }| {
                        match weight {
                            3 => format!("{position}A"),
                            2 => format!("{position}B"),
                            1 => format!("{position}C"),
                            // 'D' is the default value and does not need to be displayed
                            _ => format!("{position}"),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",");

                f.write_str(&format!("'{}':{}", word, position))?;
            }

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
            let mut positions = Vec::<LexemeMeta>::with_capacity(num_positions as usize);

            if num_positions > 0 {
                for _ in 0..num_positions {
                    let position = reader.read_u16::<BigEndian>()?;
                    positions.push(LexemeMeta::from(position));
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
                for lexeme_meta in &lexeme.positions {
                    buf.write_u16::<BigEndian>(lexeme_meta.into())?;
                }
            }
        }

        buf.flush()?;

        Ok(buf.to_vec())
    }
}

fn split_into_ts_vector_words(input: &str) -> Vec<String> {
    let mut wrapped = false;
    let mut words = vec![];
    let mut current_word = String::new();
    let mut escaped = false;

    let mut chars = input.chars().peekable();

    while let Some(token) = chars.next() {
        match token {
            '\'' => {
                if !escaped {
                    if chars.peek().is_some_and(|item| *item == '\'') {
                        escaped = true;
                        current_word += "'";
                    } else {
                        wrapped = !wrapped;
                    }
                } else {
                    escaped = false;
                }
            }
            char => {
                if char.is_whitespace() && !wrapped {
                    words.push(current_word);
                    current_word = String::new();
                } else {
                    current_word += &char.to_string();
                }
            }
        }
    }

    if !current_word.is_empty() {
        words.push(current_word);
        current_word = String::new();
    }

    words
}

impl FromStr for TsVector {
    type Err = ParseLexemeMetaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words: Vec<Lexeme> = vec![];

        for word in split_into_ts_vector_words(s) {
            if let Some((word, positions)) = word.rsplit_once(':') {
                words.push(Lexeme {
                    word: word
                        .trim_start_matches('\'')
                        .trim_end_matches('\'')
                        .to_string(),
                    positions: positions
                        .split(',')
                        .map(|value| Ok::<LexemeMeta, ParseLexemeMetaError>(value.parse()?))
                        .collect::<Result<Vec<_>, _>>()?,
                });
            } else {
                words.push(Lexeme {
                    word: word
                        .trim_start_matches('\'')
                        .trim_end_matches('\'')
                        .to_string(),
                    positions: vec![],
                })
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
