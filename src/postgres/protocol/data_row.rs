use super::{Decode, Buf};
use std::{
    convert::TryInto,
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
    fn decode(mut buf: &[u8]) -> io::Result<Self> {
        let len = buf.get_u16()? as usize;
        let buffer: Pin<Box<[u8]>> = Pin::new(buf.into());
        let mut buf = &*buffer;
        let mut values = Vec::with_capacity(len);

        while values.len() < len {
            // The length of the column value, in bytes (this count does not include itself).
            // Can be zero. As a special case, -1 indicates a NULL column value.
            // No value bytes follow in the NULL case.
            let value_len = buf.get_i32()?;

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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::{DataRow, Decode};
    use std::io;

    const DATA_ROW: &[u8] = b"\0\x03\0\0\0\x011\0\0\0\x012\0\0\0\x013";

    #[test]
    fn it_decodes_data_row() {
        let message = DataRow::decode(DATA_ROW).unwrap();

        assert_eq!(message.len(), 3);

        assert_eq!(message.get(0), Some(&b"1"[..]));
        assert_eq!(message.get(1), Some(&b"2"[..]));
        assert_eq!(message.get(2), Some(&b"3"[..]));
    }

    #[bench]
    fn bench_decode_data_row(b: &mut test::Bencher) {
        b.iter(|| DataRow::decode(DATA_ROW).unwrap());
    }
}
