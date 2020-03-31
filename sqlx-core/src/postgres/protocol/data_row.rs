use crate::io::Buf;
use crate::postgres::database::Postgres;
use byteorder::NetworkEndian;
use std::ops::Range;

pub(crate) struct DataRow<'c> {
    values: &'c [Option<(u32, u32)>],
    buffer: &'c [u8],
}

impl<'c> DataRow<'c> {
    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&'c [u8]> {
        self.values[index]
            .as_ref()
            .map(|(offset, size)| &self.buffer[(*offset as usize)..((*offset + *size) as usize)])
    }
}

impl<'c> DataRow<'c> {
    pub(crate) fn read(
        buffer: &'c [u8],
        values: &'c mut Vec<Option<(u32, u32)>>,
    ) -> crate::Result<Self> {
        values.clear();

        let mut buf = buffer;

        let len = buf.get_u16::<NetworkEndian>()?;

        let mut offset = 6;

        while values.len() < (len as usize) {
            // The length of the column value, in bytes (this count does not include itself).
            // Can be zero. As a special case, -1 indicates a NULL column value.
            // No value bytes follow in the NULL case.
            let mut size = buf.get_i32::<NetworkEndian>()?;

            if size < 0 {
                values.push(None);

                offset += 4;
            } else {
                values.push(Some((offset, size as u32)));

                offset += (size as u32) + 4;
                buf.advance(size as usize);
            }
        }

        Ok(Self { buffer, values })
    }
}

#[cfg(feature = "bench")]
#[bench]
fn bench_get_data_row(b: &mut test::Bencher) {
    let buffer = b"\x00\x08\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00\n\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00\x14\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00(\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00P";
    let mut values = Vec::with_capacity(10);
    let row = DataRow::read(buffer, &mut values).unwrap();

    b.iter(|| {
        assert_eq!(row.get(0), None);
        assert_eq!(row.get(1), Some(&[0, 0, 0, 10][..]));
        assert_eq!(row.get(2), None);
        assert_eq!(row.get(3), Some(&[0, 0, 0, 20][..]));
        assert_eq!(row.get(4), None);
    });
}

#[cfg(feature = "bench")]
#[bench]
fn bench_read_data_row(b: &mut test::Bencher) {
    let buffer = b"\x00\x08\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00\n\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00\x14\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00(\xff\xff\xff\xff\x00\x00\x00\x04\x00\x00\x00P";
    let mut values = Vec::with_capacity(10);

    b.iter(|| {
        let row = DataRow::read(buffer, &mut values).unwrap();
    });
}
