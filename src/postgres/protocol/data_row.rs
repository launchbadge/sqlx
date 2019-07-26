use super::Decode;
use bytes::Bytes;
use std::{
    convert::TryInto,
    fmt::{self, Debug},
    io,
    ops::Range,
};

pub struct DataRow {
    ranges: Vec<Option<Range<usize>>>,
    buf: Bytes,
}

impl Decode for DataRow {
    fn decode(src: Bytes) -> io::Result<Self> {
        let len = u16::from_be_bytes(src.as_ref()[..2].try_into().unwrap());

        let mut ranges = Vec::with_capacity(len as usize);
        let mut rem = len;
        let mut index = 2;

        while rem > 0 {
            // The length of the column value, in bytes (this count does not include itself).
            // Can be zero. As a special case, -1 indicates a NULL column value.
            // No value bytes follow in the NULL case.
            let value_len =
                i32::from_be_bytes(src.as_ref()[index..(index + 4)].try_into().unwrap());
            index += 4;

            if value_len == -1 {
                ranges.push(None);
            } else {
                let value_beg = index;
                let value_end = value_beg + (value_len as usize);

                ranges.push(Some(value_beg..(value_end as usize)));

                index += value_len as usize;
            }

            rem -= 1;
        }

        Ok(Self { ranges, buf: src })
    }
}

impl DataRow {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&[u8]> {
        Some(&self.buf[self.ranges[index].clone()?])
    }
}

impl Debug for DataRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DataRow")
            .field(
                "values",
                &self
                    .ranges
                    .iter()
                    .map(|range| Some(Bytes::from(&self.buf[range.clone()?])))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{DataRow, Decode};
    use bytes::Bytes;
    use std::io;

    const DATA_ROW: &[u8] = b"\0\x03\0\0\0\x011\0\0\0\x012\0\0\0\x013";

    #[test]
    fn it_decodes_data_row() -> io::Result<()> {
        let src = Bytes::from_static(DATA_ROW);
        let message = DataRow::decode(src)?;

        assert_eq!(message.len(), 3);

        assert_eq!(message.get(0), Some(&b"1"[..]));
        assert_eq!(message.get(1), Some(&b"2"[..]));
        assert_eq!(message.get(2), Some(&b"3"[..]));

        Ok(())
    }
}
