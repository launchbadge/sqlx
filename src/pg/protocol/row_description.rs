use super::Decode;
use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;
use memchr::memchr;
use std::{
    io,
    mem::size_of_val,
    num::{NonZeroI16, NonZeroU32},
    str,
};

// TODO: Custom Debug for RowDescription and FieldDescription

/// A descriptive record on a single field received from PostgreSQL.
#[derive(Debug)]
pub struct FieldDescription<'a> {
    name: &'a str,
    table_oid: Option<NonZeroU32>,
    column_attribute_num: Option<NonZeroI16>,
    type_oid: u32,
    type_size: i16,
    type_modifier: i32,
    format: i16,
}

impl<'a> FieldDescription<'a> {
    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }

    #[inline]
    pub fn table_oid(&self) -> Option<u32> {
        self.table_oid.map(Into::into)
    }

    #[inline]
    pub fn column_attribute_num(&self) -> Option<i16> {
        self.column_attribute_num.map(Into::into)
    }

    #[inline]
    pub fn type_oid(&self) -> u32 {
        self.type_oid
    }

    #[inline]
    pub fn type_size(&self) -> i16 {
        self.type_size
    }

    #[inline]
    pub fn type_modifier(&self) -> i32 {
        self.type_modifier
    }

    #[inline]
    pub fn format(&self) -> i16 {
        self.format
    }
}

#[derive(Debug)]
pub struct RowDescription {
    // The number of fields in a row (can be zero).
    len: u16,
    data: Vec<u8>,
}

impl RowDescription {
    pub fn fields(&self) -> FieldDescriptions<'_> {
        FieldDescriptions {
            rem: self.len,
            buf: &self.data,
        }
    }
}

impl Decode for RowDescription {
    fn decode(src: &[u8]) -> io::Result<Self> {
        let len = BigEndian::read_u16(&src[..2]);

        Ok(Self {
            len,
            data: src[2..].into(),
        })
    }
}

pub struct FieldDescriptions<'a> {
    rem: u16,
    buf: &'a [u8],
}

impl<'a> Iterator for FieldDescriptions<'a> {
    type Item = FieldDescription<'a>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.rem as usize, Some(self.rem as usize))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem == 0 {
            return None;
        }

        let name_end = memchr(0, &self.buf).unwrap();
        let mut idx = name_end + 1;
        let name = unsafe { str::from_utf8_unchecked(&self.buf[..name_end]) };

        let table_oid = BigEndian::read_u32(&self.buf[idx..]);
        idx += size_of_val(&table_oid);

        let column_attribute_num = BigEndian::read_i16(&self.buf[idx..]);
        idx += size_of_val(&column_attribute_num);

        let type_oid = BigEndian::read_u32(&self.buf[idx..]);
        idx += size_of_val(&type_oid);

        let type_size = BigEndian::read_i16(&self.buf[idx..]);
        idx += size_of_val(&type_size);

        let type_modifier = BigEndian::read_i32(&self.buf[idx..]);
        idx += size_of_val(&type_modifier);

        let format = BigEndian::read_i16(&self.buf[idx..]);
        idx += size_of_val(&format);

        self.rem -= 1;
        self.buf = &self.buf[idx..];

        Some(FieldDescription {
            name,
            table_oid: NonZeroU32::new(table_oid),
            column_attribute_num: NonZeroI16::new(column_attribute_num),
            type_oid,
            type_size,
            type_modifier,
            format,
        })
    }
}

impl<'a> ExactSizeIterator for FieldDescriptions<'a> {}

#[cfg(test)]
mod tests {
    use super::{Decode, RowDescription};
    use bytes::Bytes;
    use std::io;

    const ROW_DESC: &[u8] = b"\0\x03?column?\0\0\0\0\0\0\0\0\0\0\x17\0\x04\xff\xff\xff\xff\0\0?column?\0\0\0\0\0\0\0\0\0\0\x17\0\x04\xff\xff\xff\xff\0\0?column?\0\0\0\0\0\0\0\0\0\0\x17\0\x04\xff\xff\xff\xff\0\0";

    #[test]
    fn it_decodes_row_description() -> io::Result<()> {
        let message = RowDescription::decode(ROW_DESC)?;
        assert_eq!(message.fields().len(), 3);

        for field in message.fields() {
            assert_eq!(field.name(), "?column?");
            assert_eq!(field.table_oid(), None);
            assert_eq!(field.column_attribute_num(), None);
            assert_eq!(field.type_oid(), 23);
            assert_eq!(field.type_size(), 4);
            assert_eq!(field.type_modifier(), -1);
            assert_eq!(field.format(), 0);
        }

        Ok(())
    }
}
