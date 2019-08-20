use super::Decode;
use bytes::Bytes;
use std::{
    convert::TryInto,
    fmt::{self, Debug},
    io,
    ops::Range,
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
    fn decode(buf: &[u8]) -> Self {
        let buffer: Pin<Box<[u8]>> = Pin::new(buf.into());

        let len_b: [u8; 2] = buffer[..2].try_into().unwrap();
        let len = u16::from_be_bytes(len_b) as usize;

        let mut values = Vec::with_capacity(len);
        let mut index = 2;

        while values.len() < len {
            // The length of the column value, in bytes (this count does not include itself).
            // Can be zero. As a special case, -1 indicates a NULL column value.
            // No value bytes follow in the NULL case.
            // TODO: Handle unwrap
            let value_len_b: [u8; 4] = buffer[index..(index + 4)].try_into().unwrap();
            let value_len = i32::from_be_bytes(value_len_b);
            index += 4;

            if value_len == -1 {
                values.push(None);
            } else {
                let value_len = value_len as usize;
                let value = &buffer[index..(index + value_len)];
                index += value_len as usize;

                values.push(Some(value.into()));
            }
        }

        Self {
            values: values.into_boxed_slice(),
            buffer,
        }
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
    use bytes::Bytes;
    use std::io;

    const DATA_ROW: &[u8] = b"\0\x03\0\0\0\x011\0\0\0\x012\0\0\0\x013";

    #[test]
    fn it_decodes_data_row() {
        let message = DataRow::decode(DATA_ROW);

        assert_eq!(message.len(), 3);

        assert_eq!(message.get(0), Some(&b"1"[..]));
        assert_eq!(message.get(1), Some(&b"2"[..]));
        assert_eq!(message.get(2), Some(&b"3"[..]));
    }
}
