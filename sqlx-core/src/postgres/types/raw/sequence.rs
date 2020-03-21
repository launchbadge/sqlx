use crate::decode::DecodeOwned;
use crate::io::Buf;
use crate::postgres::{PgValue, Postgres};
use crate::types::Type;
use byteorder::BigEndian;

pub(crate) struct PgSequenceDecoder<'de> {
    value: PgValue<'de>,
    len: usize,
    mixed: bool,
}

impl<'de> PgSequenceDecoder<'de> {
    pub(crate) fn new(mut value: PgValue<'de>, mixed: bool) -> Self {
        match value {
            PgValue::Binary(_) => {
                // assume that this has already gotten tweaked by the caller as
                // tuples and arrays have a very different header
            }

            PgValue::Text(ref mut s) => {
                // remove the outer ( ... ) or { ... }
                *s = &s[1..(s.len() - 1)];
            }
        }

        Self {
            value,
            mixed,
            len: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn decode<T>(&mut self) -> crate::Result<Postgres, Option<T>>
    where
        T: DecodeOwned<Postgres>,
        T: Type<Postgres>,
    {
        match self.value {
            PgValue::Binary(ref mut buf) => {
                if buf.is_empty() {
                    return Ok(None);
                }

                // mixed sequences can contain values of many different types
                // the OID of the type is encoded next to each value
                if self.mixed {
                    // TODO: We should fail if this type is not _compatible_; but
                    //       I want to make sure we handle this _and_ the outer level
                    //       type mismatch errors at the same time

                    let _oid = buf.get_u32::<BigEndian>()?;
                }

                let len = buf.get_i32::<BigEndian>()? as isize;

                let value = if len < 0 {
                    T::decode(None)?
                } else {
                    let value_buf = &buf[..(len as usize)];

                    *buf = &buf[(len as usize)..];

                    T::decode(Some(PgValue::Binary(value_buf)))?
                };

                self.len += 1;

                Ok(Some(value))
            }

            PgValue::Text(ref mut s) => {
                if s.is_empty() {
                    return Ok(None);
                }

                let mut value = String::new();
                let mut in_quotes = false;
                let mut in_escape = false;
                let mut in_maybe_quote_escape = false;

                let end: Option<usize> = 'outer: loop {
                    let mut iter = s.char_indices().peekable();
                    while let Some((index, ch)) = iter.next() {
                        if in_maybe_quote_escape {
                            if ch == '"' {
                                // double quote escape
                                value.push('"');
                                in_maybe_quote_escape = false;
                                continue;
                            } else {
                                // that was actually a quote
                                in_quotes = !in_quotes;
                            }
                        }

                        match ch {
                            ',' if !in_quotes => break 'outer Some(index),

                            '\\' if !in_escape => {
                                in_escape = true;
                            }

                            _ if in_escape => {
                                // special escape sequences only matter for string parsing
                                // we only will ever receive stuff like "\\b" that we translate
                                // to "\b"
                                value.push(ch);

                                // skip prev_ch assignment for
                                //an escape sequence resolution
                                in_escape = false;
                                continue;
                            }

                            '"' if in_quotes => {
                                in_maybe_quote_escape = true;
                            }

                            '"' => {
                                in_quotes = !in_quotes;
                            }

                            _ => value.push(ch),
                        }
                    }

                    // Reached the end of the string
                    break None;
                };

                let value = T::decode(if end == Some(0) {
                    None
                } else if !self.mixed && value == "NULL" {
                    // Yes, in arrays the text encoding of a NULL is just NULL
                    None
                } else {
                    Some(PgValue::Text(&value))
                })?;

                *s = if let Some(end) = end {
                    &s[end + 1..]
                } else {
                    ""
                };

                self.len += 1;

                Ok(Some(value))
            }
        }
    }
}

impl<'de> From<&'de str> for PgSequenceDecoder<'de> {
    fn from(s: &'de str) -> Self {
        Self::new(PgValue::Text(s), false)
    }
}

#[cfg(test)]
mod tests {
    use super::PgSequenceDecoder;
    use crate::postgres::Postgres;

    #[test]
    fn it_decodes_text_number() -> crate::Result<Postgres, ()> {
        // select (10,20,-220);
        let data = "(10,20,-220)";
        let mut decoder = PgSequenceDecoder::from(data);

        assert_eq!(decoder.decode::<i32>()?, Some(10_i32));
        assert_eq!(decoder.decode::<i32>()?, Some(20_i32));
        assert_eq!(decoder.decode::<i32>()?, Some(-220_i32));
        assert_eq!(decoder.decode::<i32>()?, None);

        Ok(())
    }

    #[test]
    fn it_decodes_text_nested_sequence() -> crate::Result<Postgres, ()> {
        // select ((1,array[false,true]),array[(1,4),(5,2)]);
        let data = r#"("(1,""{f,t}"")","{""(1,4)"",""(5,2)""}")"#;
        let mut decoder = PgSequenceDecoder::from(data);

        assert_eq!(
            decoder.decode::<(i32, Vec<bool>)>()?,
            Some((1, vec![false, true]))
        );

        assert_eq!(
            decoder.decode::<Vec<(i32, i32)>>()?,
            Some(vec![(1_i32, 4_i32), (5_i32, 2_i32),])
        );

        Ok(())
    }
}
