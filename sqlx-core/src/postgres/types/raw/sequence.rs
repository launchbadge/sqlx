use crate::decode::Decode;
use crate::io::Buf;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgData, PgTypeInfo, PgValue, Postgres};
use crate::types::{Type, TypeInfo};
use byteorder::BigEndian;

pub(crate) struct PgSequenceDecoder<'de> {
    data: PgData<'de>,
    len: usize,
    mixed: bool,
}

impl<'de> PgSequenceDecoder<'de> {
    pub(crate) fn new(mut data: PgData<'de>, mixed: bool) -> Self {
        match data {
            PgData::Binary(_) => {
                // assume that this has already gotten tweaked by the caller as
                // tuples and arrays have a very different header
            }

            PgData::Text(ref mut s) => {
                // remove the outer ( ... ) or { ... }
                *s = &s[1..(s.len() - 1)];
            }
        }

        Self {
            data,
            mixed,
            len: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn decode<T>(&mut self) -> crate::Result<Option<T>>
    where
        T: for<'seq> Decode<'seq, Postgres>,
        T: Type<Postgres>,
    {
        match self.data {
            PgData::Binary(ref mut buf) => {
                if buf.is_empty() {
                    return Ok(None);
                }

                // mixed sequences can contain values of many different types
                // the OID of the type is encoded next to each value
                let type_id = if self.mixed {
                    let oid = buf.get_u32::<BigEndian>()?;
                    let expected_ty = PgTypeInfo::with_oid(oid);

                    if !expected_ty.compatible(&T::type_info()) {
                        return Err(crate::Error::mismatched_types::<Postgres, T>(expected_ty));
                    }

                    TypeId(oid)
                } else {
                    // NOTE: We don't validate the element type for non-mixed sequences because
                    //       the outer type like `text[]` would have already ensured we are dealing
                    //       with a Vec<String>
                    T::type_info().id
                };

                let len = buf.get_i32::<BigEndian>()? as isize;

                let value = if len < 0 {
                    T::decode(PgValue::null(type_id))?
                } else {
                    let value_buf = &buf[..(len as usize)];

                    *buf = &buf[(len as usize)..];

                    T::decode(PgValue::bytes(type_id, value_buf))?
                };

                self.len += 1;

                Ok(Some(value))
            }

            PgData::Text(ref mut s) => {
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

                // NOTE: We pass `0` as the type ID because we don't have a reasonable value
                //       we could use. In TEXT mode, sequences aren't typed.

                let value = T::decode(if end == Some(0) {
                    PgValue::null(TypeId(0))
                } else if !self.mixed && value == "NULL" {
                    // Yes, in arrays the text encoding of a NULL is just NULL
                    PgValue::null(TypeId(0))
                } else {
                    PgValue::str(TypeId(0), &*value)
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
        Self::new(PgData::Text(s), false)
    }
}

#[cfg(test)]
mod tests {
    use super::PgSequenceDecoder;

    #[test]
    fn it_decodes_text_number() -> crate::Result<()> {
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
    fn it_decodes_text_nested_sequence() -> crate::Result<()> {
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
