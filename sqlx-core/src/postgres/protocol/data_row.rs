use crate::io::{Buf, ByteStr};
use crate::postgres::protocol::Decode;
use byteorder::NetworkEndian;
use std::fmt::{self, Debug};
use std::ops::Range;

pub struct DataRow {
    buffer: Box<[u8]>,
    values: Box<[Option<Range<u32>>]>,
}

impl DataRow {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> Option<&[u8]> {
        let range = self.values[index].as_ref()?;

        Some(&self.buffer[(range.start as usize)..(range.end as usize)])
    }
}

impl Decode for DataRow {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let len = buf.get_u16::<NetworkEndian>()? as usize;
        let buffer: Box<[u8]> = buf.into();
        let mut values = Vec::with_capacity(len);
        let mut index = 4;

        while values.len() < len {
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
            values: values.into_boxed_slice(),
            buffer,
        })
    }
}

impl Debug for DataRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataRow(")?;

        let len = self.values.len();

        f.debug_list()
            .entries((0..len).map(|i| self.get(i).map(ByteStr)))
            .finish()?;

        write!(f, ")")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DataRow, Decode};

    const DATA_ROW: &[u8] = b"\0\x03\0\0\0\x011\0\0\0\x012\0\0\0\x013";

    #[test]
    fn it_decodes_data_row() {
        let m = DataRow::decode(DATA_ROW).unwrap();

        assert_eq!(m.values.len(), 3);

        assert_eq!(m.get(0), Some(&b"1"[..]));
        assert_eq!(m.get(1), Some(&b"2"[..]));
        assert_eq!(m.get(2), Some(&b"3"[..]));

        assert_eq!(
            format!("{:?}", m),
            "DataRow([Some(b\"1\"), Some(b\"2\"), Some(b\"3\")])"
        );
    }
}
