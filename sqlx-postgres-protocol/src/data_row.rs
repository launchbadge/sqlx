use crate::Decode;
use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;
use std::io;

// TODO: Custom Debug for DataRow

#[derive(Debug)]
pub struct DataRow {
    len: u16,
    data: Bytes,
}

impl DataRow {
    pub fn values(&self) -> DataValues<'_> {
        DataValues {
            rem: self.len,
            buf: &*self.data,
        }
    }
}

impl Decode for DataRow {
    fn decode(src: Bytes) -> io::Result<Self> {
        let len = BigEndian::read_u16(&src[..2]);

        Ok(Self {
            len,
            data: src.slice_from(2),
        })
    }
}

pub struct DataValues<'a> {
    rem: u16,
    buf: &'a [u8],
}

impl<'a> Iterator for DataValues<'a> {
    type Item = Option<&'a [u8]>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.rem as usize, Some(self.rem as usize))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem == 0 {
            return None;
        }

        let len = BigEndian::read_i32(self.buf);
        let size = (if len < 0 { 0 } else { len }) as usize;

        let value = if len == -1 {
            None
        } else {
            Some(&self.buf[4..(4 + len) as usize])
        };

        self.rem -= 1;
        self.buf = &self.buf[(size + 4)..];

        Some(value)
    }
}

impl<'a> ExactSizeIterator for DataValues<'a> {}

#[cfg(test)]
mod tests {
    use super::DataRow;
    use crate::Decode;
    use bytes::Bytes;
    use std::io;

    const DATA_ROW: &[u8] = b"\0\x03\0\0\0\x011\0\0\0\x012\0\0\0\x013";

    #[test]
    fn it_decodes_data_row() -> io::Result<()> {
        let src = Bytes::from_static(DATA_ROW);
        let message = DataRow::decode(src)?;
        assert_eq!(message.values().len(), 3);

        for (index, value) in message.values().enumerate() {
            // "1", "2", "3"
            assert_eq!(value, Some(&[(index + 1 + 48) as u8][..]));
        }

        Ok(())
    }
}
