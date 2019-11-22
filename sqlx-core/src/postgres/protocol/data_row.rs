use super::Decode;
use crate::io::{Buf, ByteStr};
use byteorder::NetworkEndian;
use std::{
    fmt::{self, Debug},
    io,
    pin::Pin,
    ptr::NonNull,
};

pub struct DataRow {
    #[used]
    buffer: Pin<Box<[u8]>>,
    values: Box<[Option<NonNull<[u8]>>]>,
}

// SAFE: Raw pointers point to pinned memory inside the struct
unsafe impl Send for DataRow {}
unsafe impl Sync for DataRow {}

impl Decode for DataRow {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let cnt = buf.get_u16::<NetworkEndian>()? as usize;
        let buffer: Pin<Box<[u8]>> = Pin::new(buf.into());
        let mut buf = &*buffer;
        let mut values = Vec::with_capacity(cnt);

        while values.len() < cnt {
            // The length of the column value, in bytes (this count does not include itself).
            // Can be zero. As a special case, -1 indicates a NULL column value.
            // No value bytes follow in the NULL case.
            let value_len = buf.get_i32::<NetworkEndian>()?;

            if value_len == -1 {
                values.push(None);
            } else {
                values.push(Some(buf[..(value_len as usize)].into()));
                buf.advance(value_len as usize);
            }
        }

        Ok(Self {
            values: values.into_boxed_slice(),
            buffer,
        })
    }
}

impl DataRow {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&[u8]> {
        self.values[index]
            .as_ref()
            .map(|value| unsafe { value.as_ref() })
    }
}

impl Debug for DataRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataRow(")?;

        f.debug_list()
            .entries((0..self.len()).map(|i| self.get(i).map(ByteStr)))
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

        assert_eq!(m.len(), 3);

        assert_eq!(m.get(0), Some(&b"1"[..]));
        assert_eq!(m.get(1), Some(&b"2"[..]));
        assert_eq!(m.get(2), Some(&b"3"[..]));

        assert_eq!(
            format!("{:?}", m),
            "DataRow([Some(b\"1\"), Some(b\"2\"), Some(b\"3\")])"
        );
    }
}
