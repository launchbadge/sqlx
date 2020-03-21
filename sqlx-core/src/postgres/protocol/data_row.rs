use crate::io::Buf;
use crate::postgres::database::Postgres;
use byteorder::NetworkEndian;
use std::ops::Range;

pub(crate) struct DataRow<'c> {
    len: u16,
    buffer: &'c [u8],
    values: &'c [Option<Range<u32>>],
}

impl<'c> DataRow<'c> {
    pub(crate) fn len(&self) -> usize {
        self.len as usize
    }

    pub(crate) fn get(&self, index: usize) -> Option<&'c [u8]> {
        let range = self.values[index].as_ref()?;

        Some(&self.buffer[(range.start as usize)..(range.end as usize)])
    }
}

impl<'c> DataRow<'c> {
    pub(crate) fn read(
        buffer: &'c [u8],
        values: &'c mut Vec<Option<Range<u32>>>,
    ) -> crate::Result<Postgres, Self> {
        // let buffer = connection.stream.buffer();
        // let values = &mut connection.current_row_values;

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

        Ok(Self {
            len,
            buffer,
            values,
        })
    }
}
