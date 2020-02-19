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
    pub(crate) fn read<'a>(
        connection: &mut PgConnection,
        // buffer: &'a [u8],
        // values: &'a mut Vec<Option<Range<u32>>>,
    ) -> crate::Result<Self> {
        let buffer = connection.stream.buffer();
        let values = &mut connection.data_row_values_buf;

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

#[cfg(test)]
mod tests {
    use super::{DataRow, Decode};

    const DATA_ROW: &[u8] = b"\0\x03\0\0\0\x011\0\0\0\x012\0\0\0\x013";

    #[test]
    fn it_reads_data_row() {
        let mut values = Vec::new();
        let m = DataRow::read(DATA_ROW, &mut values).unwrap();

        assert_eq!(m.len, 3);

        assert_eq!(m.get(DATA_ROW, &values, 0), Some(&b"1"[..]));
        assert_eq!(m.get(DATA_ROW, &values, 1), Some(&b"2"[..]));
        assert_eq!(m.get(DATA_ROW, &values, 2), Some(&b"3"[..]));
    }
}
