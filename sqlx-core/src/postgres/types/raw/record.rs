use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::io::Buf;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgTypeInfo, PgValue, Postgres};
use crate::types::Type;
use byteorder::BigEndian;
use std::convert::TryInto;

pub struct PgRecordEncoder<'a> {
    buf: &'a mut Vec<u8>,
    beg: usize,
    num: u32,
}

impl<'a> PgRecordEncoder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        // reserve space for a field count
        buf.extend_from_slice(&(0_u32).to_be_bytes());

        Self {
            beg: buf.len(),
            buf,
            num: 0,
        }
    }

    pub fn finish(&mut self) {
        // replaces zeros with actual length
        self.buf[self.beg - 4..self.beg].copy_from_slice(&self.num.to_be_bytes());
    }

    pub fn encode<T>(&mut self, value: T) -> &mut Self
    where
        T: Type<Postgres> + Encode<Postgres>,
    {
        // write oid
        let info = T::type_info();
        self.buf.extend(&info.oid().to_be_bytes());

        // write zeros for length
        self.buf.extend(&[0; 4]);

        let start = self.buf.len();
        if let IsNull::Yes = value.encode_nullable(self.buf) {
            // replaces zeros with actual length
            self.buf[start - 4..start].copy_from_slice(&(-1_i32).to_be_bytes());
        } else {
            let end = self.buf.len();
            let size = end - start;

            // replaces zeros with actual length
            self.buf[start - 4..start].copy_from_slice(&(size as u32).to_be_bytes());
        }

        // keep track of count
        self.num += 1;

        self
    }
}

pub struct PgRecordDecoder<'de> {
    value: PgValue<'de>,
}

impl<'de> PgRecordDecoder<'de> {
    pub fn new(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        let mut value: PgValue = value.try_into()?;

        match value {
            PgValue::Binary(ref mut buf) => {
                let _expected_len = buf.get_u32::<BigEndian>()?;
            }

            PgValue::Text(ref mut s) => {
                // remove outer ( ... )
                *s = &s[1..(s.len() - 1)];
            }
        }

        Ok(Self { value })
    }

    pub fn decode<T>(&mut self) -> crate::Result<T>
    where
        T: Decode<'de, Postgres>,
    {
        match self.value {
            PgValue::Binary(ref mut buf) => {
                // TODO: We should fail if this type is not _compatible_; but
                //       I want to make sure we handle this _and_ the outer level
                //       type mismatch errors at the same time
                let _oid = buf.get_u32::<BigEndian>()?;
                let len = buf.get_i32::<BigEndian>()? as isize;

                let value = if len < 0 {
                    T::decode(None)?
                } else {
                    let value_buf = &buf[..(len as usize)];
                    *buf = &buf[(len as usize)..];

                    T::decode(Some(PgValue::Binary(value_buf)))?
                };

                Ok(value)
            }

            PgValue::Text(ref mut s) => {
                let mut in_quotes = false;
                let mut in_escape = false;
                let mut is_quoted = false;
                let mut prev_ch = '\0';
                let mut eos = false;
                let mut prev_index = 0;
                let mut value = String::new();

                let index = 'outer: loop {
                    let mut iter = s.char_indices();
                    while let Some((index, ch)) = iter.next() {
                        match ch {
                            ',' if !in_quotes => {
                                break 'outer Some(prev_index);
                            }

                            ',' if prev_ch == '\0' => {
                                break 'outer None;
                            }

                            '"' if prev_ch == '"' && index != 1 => {
                                // Quotes are escaped with another quote
                                in_quotes = false;
                                value.push('"');
                            }

                            '"' if in_quotes => {
                                in_quotes = false;
                            }

                            '\'' if in_escape => {
                                in_escape = false;
                                value.push('\'');
                            }

                            '"' if in_escape => {
                                in_escape = false;
                                value.push('"');
                            }

                            '\\' if in_escape => {
                                in_escape = false;
                                value.push('\\');
                            }

                            '\\' => {
                                in_escape = true;
                            }

                            '"' => {
                                is_quoted = true;
                                in_quotes = true;
                            }

                            ch => {
                                value.push(ch);
                            }
                        }

                        prev_index = index;
                        prev_ch = ch;
                    }

                    eos = true;

                    break 'outer if prev_ch == '\0' {
                        // NULL values have zero characters
                        // Empty strings are ""
                        None
                    } else {
                        Some(prev_index)
                    };
                };

                let value = index.map(|index| {
                    let mut s = &s[..=index];

                    if is_quoted {
                        s = &s[1..s.len() - 1];
                    }

                    PgValue::Text(s)
                });

                let value = T::decode(value)?;

                if !eos {
                    *s = &s[index.unwrap_or(0) + 2..];
                } else {
                    *s = "";
                }

                Ok(value)
            }
        }
    }
}

macro_rules! impl_pg_record_for_tuple {
    ($( $idx:ident : $T:ident ),+) => {
        impl<$($T,)+> Type<Postgres> for ($($T,)+) {
            #[inline]
            fn type_info() -> PgTypeInfo {
                PgTypeInfo {
                    id: TypeId(2249),
                    name: Some("RECORD".into()),
                }
            }
        }

        impl<'de, $($T,)+> Decode<'de, Postgres> for ($($T,)+)
        where
            $($T: crate::types::Type<Postgres>,)+
            $($T: crate::decode::Decode<'de, Postgres>,)+
        {
            fn decode(value: Option<PgValue<'de>>) -> crate::Result<Self> {
                let mut decoder = PgRecordDecoder::new(value)?;

                $(let $idx: $T = decoder.decode()?;)+

                Ok(($($idx,)+))
            }
        }
    };
}

impl_pg_record_for_tuple!(_1: T1);

impl_pg_record_for_tuple!(_1: T1, _2: T2);

impl_pg_record_for_tuple!(_1: T1, _2: T2, _3: T3);

impl_pg_record_for_tuple!(_1: T1, _2: T2, _3: T3, _4: T4);

impl_pg_record_for_tuple!(_1: T1, _2: T2, _3: T3, _4: T4, _5: T5);

impl_pg_record_for_tuple!(_1: T1, _2: T2, _3: T3, _4: T4, _5: T5, _6: T6);

impl_pg_record_for_tuple!(_1: T1, _2: T2, _3: T3, _4: T4, _5: T5, _6: T6, _7: T7);

impl_pg_record_for_tuple!(
    _1: T1,
    _2: T2,
    _3: T3,
    _4: T4,
    _5: T5,
    _6: T6,
    _7: T7,
    _8: T8
);

impl_pg_record_for_tuple!(
    _1: T1,
    _2: T2,
    _3: T3,
    _4: T4,
    _5: T5,
    _6: T6,
    _7: T7,
    _8: T8,
    _9: T9
);

#[test]
fn test_encode_field() {
    let value = "Foo Bar";
    let mut raw_encoded = Vec::new();
    <&str as Encode<Postgres>>::encode(&value, &mut raw_encoded);
    let mut field_encoded = Vec::new();

    let mut encoder = PgRecordEncoder::new(&mut field_encoded);
    encoder.encode(&value);

    // check oid
    let oid = <&str as Type<Postgres>>::type_info().oid();
    let field_encoded_oid = u32::from_be_bytes(field_encoded[4..8].try_into().unwrap());
    assert_eq!(oid, field_encoded_oid);

    // check length
    let field_encoded_length = u32::from_be_bytes(field_encoded[8..12].try_into().unwrap());
    assert_eq!(raw_encoded.len(), field_encoded_length as usize);

    // check data
    assert_eq!(raw_encoded, &field_encoded[12..]);
}

#[test]
fn test_decode_field() {
    let value = "Foo Bar".to_string();

    let mut buf = Vec::new();
    let mut encoder = PgRecordEncoder::new(&mut buf);
    encoder.encode(&value);

    let mut buf = buf.as_slice();
    let mut decoder = PgRecordDecoder::new(Some(PgValue::Binary(buf))).unwrap();

    let value_decoded: String = decoder.decode().unwrap();
    assert_eq!(value_decoded, value);
}
