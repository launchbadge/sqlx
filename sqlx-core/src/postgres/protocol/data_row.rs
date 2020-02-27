use crate::io::{Buf, ByteStr};
use crate::postgres::protocol::Decode;
use crate::postgres::PgConnection;
use byteorder::NetworkEndian;
use std::fmt::{self, Debug};
use std::ops::Range;

pub struct DataRow {
    len: u16,
}

impl DataRow {
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn get<'a>(
        &self,
        buffer: &'a [u8],
        values: &[Option<Range<u32>>],
        index: usize,
    ) -> Option<&'a [u8]> {
        let range = values[index].as_ref()?;

        Some(&buffer[(range.start as usize)..(range.end as usize)])
    }
}

impl DataRow {
    pub(crate) fn read(connection: &mut PgConnection) -> crate::Result<Self> {
        let buffer = connection.stream.buffer();
        let values = &mut connection.current_row_values;

        values.clear();

        let mut buf = buffer;

        let len = buf.get_u16::<NetworkEndian>()?;

        let mut index = 6;

        while values.len() < (len as usize) {
            // The length of the column value, in bytes (this count does not include itself).
            // Can be zero. As a special case, -1 indicates a NULL column value.
            // No value bytes follow in the NULL case.
            let size = buf.get_i32::<NetworkEndian>()?;

            if size == -1 {
                values.push(None);

                index += 4;
            } else {
                values.push(Some((index)..(index + (size as u32))));

                index += (size as u32) + 4;
                buf.advance(size as usize);
            }
        }

        Ok(Self { len })
    }
}
