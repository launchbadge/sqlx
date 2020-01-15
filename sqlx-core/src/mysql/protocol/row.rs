use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};

use crate::io::Buf;
use crate::mysql::io::BufExt;
use crate::mysql::protocol::{Decode, TypeId};

pub struct Row {
    buffer: Box<[u8]>,
    values: Box<[Option<Range<usize>>]>,
    binary: bool,
}

impl Row {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> Option<&[u8]> {
        let range = self.values[index].as_ref()?;

        Some(&self.buffer[(range.start as usize)..(range.end as usize)])
    }
}

fn get_lenenc(buf: &[u8]) -> usize {
    match buf[0] {
        0xFB => 1,

        0xFC => {
            let len_size = 1 + 2;
            let len = LittleEndian::read_u16(&buf[1..]);

            len_size + len as usize
        }

        0xFD => {
            let len_size = 1 + 3;
            let len = LittleEndian::read_u24(&buf[1..]);

            len_size + len as usize
        }

        0xFE => {
            let len_size = 1 + 8;
            let len = LittleEndian::read_u64(&buf[1..]);

            len_size + len as usize
        }

        value => 1 + value as usize,
    }
}

impl Row {
    pub fn decode(mut buf: &[u8], columns: &[TypeId], binary: bool) -> crate::Result<Self> {
        if !binary {
            let buffer: Box<[u8]> = buf.into();
            let mut values = Vec::with_capacity(columns.len());
            let mut index = 0;

            for column_idx in 0..columns.len() {
                let size = get_lenenc(&buf[index..]);

                values.push(Some(index..(index + size)));

                index += size;
                buf.advance(size);
            }

            return Ok(Self {
                buffer,
                values: values.into_boxed_slice(),
                binary,
            });
        }

        // 0x00 header : byte<1>
        let header = buf.get_u8()?;
        if header != 0 {
            return Err(protocol_err!("expected ROW (0x00), got: {:#04X}", header).into());
        }

        // NULL-Bitmap : byte<(number_of_columns + 9) / 8>
        let null_len = (columns.len() + 9) / 8;
        let null_bitmap = &buf[..];
        buf.advance(null_len);

        let buffer: Box<[u8]> = buf.into();
        let mut values = Vec::with_capacity(columns.len());
        let mut index = 0;

        for column_idx in 0..columns.len() {
            if null_bitmap[column_idx / 8] & (1 << (column_idx % 8) as u8) != 0 {
                values.push(None);
            } else {
                let size = match columns[column_idx] {
                    TypeId::TINY_INT => 1,
                    TypeId::SMALL_INT => 2,
                    TypeId::INT => 4,
                    TypeId::BIG_INT => 8,

                    TypeId::DATE => 5,
                    TypeId::TIME => 1 + buffer[index] as usize,

                    TypeId::TIMESTAMP | TypeId::DATETIME => 1 + buffer[index] as usize,

                    TypeId::TINY_BLOB
                    | TypeId::MEDIUM_BLOB
                    | TypeId::LONG_BLOB
                    | TypeId::CHAR
                    | TypeId::TEXT
                    | TypeId::VAR_CHAR => get_lenenc(&buffer[index..]),

                    id => {
                        unimplemented!("encountered unknown field type id: {:?}", id);
                    }
                };

                values.push(Some(index..(index + size)));
                index += size;
            }
        }

        Ok(Self {
            buffer,
            values: values.into_boxed_slice(),
            binary,
        })
    }
}
