use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};

use crate::io::Buf;
use crate::mysql::protocol::TypeId;
use crate::mysql::MySqlTypeInfo;

pub(crate) struct Row<'c> {
    buffer: &'c [u8],
    values: &'c [Option<Range<usize>>],
    pub(crate) columns: &'c [MySqlTypeInfo],
    pub(crate) binary: bool,
}

impl<'c> Row<'c> {
    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&'c [u8]> {
        let range = self.values[index].as_ref()?;

        Some(&self.buffer[(range.start as usize)..(range.end as usize)])
    }
}

fn get_lenenc(buf: &[u8]) -> (usize, Option<usize>) {
    match buf[0] {
        0xFB => (1, None),

        0xFC => {
            let len_size = 1 + 2;
            let len = LittleEndian::read_u16(&buf[1..]);

            (len_size, Some(len as usize))
        }

        0xFD => {
            let len_size = 1 + 3;
            let len = LittleEndian::read_u24(&buf[1..]);

            (len_size, Some(len as usize))
        }

        0xFE => {
            let len_size = 1 + 8;
            let len = LittleEndian::read_u64(&buf[1..]);

            (len_size, Some(len as usize))
        }

        len => (1, Some(len as usize)),
    }
}

impl<'c> Row<'c> {
    pub(crate) fn read(
        mut buf: &'c [u8],
        columns: &'c [MySqlTypeInfo],
        values: &'c mut Vec<Option<Range<usize>>>,
        binary: bool,
    ) -> crate::Result<Self> {
        let buffer = &*buf;

        values.clear();
        values.reserve(columns.len());

        if !binary {
            let mut index = 0;

            for _ in 0..columns.len() {
                let (len_size, size) = get_lenenc(&buf[index..]);

                if let Some(size) = size {
                    values.push(Some((index + len_size)..(index + len_size + size)));
                } else {
                    values.push(None);
                }

                index += len_size + size.unwrap_or_default();
            }

            return Ok(Self {
                buffer,
                columns,
                values: &*values,
                binary: false,
            });
        }

        // 0x00 header : byte<1>
        let header = buf.get_u8()?;
        if header != 0 {
            return Err(protocol_err!("expected ROW (0x00), got: {:#04X}", header).into());
        }

        // NULL-Bitmap : byte<(number_of_columns + 9) / 8>
        let null_len = (columns.len() + 9) / 8;
        let null_bitmap = &buf[..];
        buf.advance(null_len);

        let buffer: Box<[u8]> = buf.into();
        let mut index = 0;

        for column_idx in 0..columns.len() {
            // the null index for a column starts at the 3rd bit in the null bitmap
            // for no reason at all besides mysql probably
            let column_null_idx = column_idx + 2;
            let is_null =
                null_bitmap[column_null_idx / 8] & (1 << (column_null_idx % 8) as u8) != 0;

            if is_null {
                values.push(None);
            } else {
                let (offset, size) = match columns[column_idx].id {
                    TypeId::TINY_INT => (0, 1),
                    TypeId::SMALL_INT => (0, 2),
                    TypeId::INT | TypeId::FLOAT => (0, 4),
                    TypeId::BIG_INT | TypeId::DOUBLE => (0, 8),

                    TypeId::DATE => (0, 5),
                    TypeId::TIME => (0, 1 + buffer[index] as usize),

                    TypeId::TIMESTAMP | TypeId::DATETIME => (0, 1 + buffer[index] as usize),

                    TypeId::TINY_BLOB
                    | TypeId::MEDIUM_BLOB
                    | TypeId::LONG_BLOB
                    | TypeId::CHAR
                    | TypeId::TEXT
                    | TypeId::ENUM
                    | TypeId::VAR_CHAR => {
                        let (len_size, len) = get_lenenc(&buffer[index..]);

                        (len_size, len.unwrap_or_default())
                    }

                    id => {
                        unimplemented!("encountered unknown field type id: {:?}", id);
                    }
                };

                values.push(Some((index + offset)..(index + offset + size)));
                index += size + offset;
            }
        }

        Ok(Self {
            buffer: buf,
            values: &*values,
            columns,
            binary,
        })
    }
}

// #[cfg(test)]
// mod test {
//     use super::super::column_count::ColumnCount;
//     use super::super::column_def::ColumnDefinition;
//     use super::super::eof::EofPacket;
//     use super::*;
//
//     #[test]
//     fn null_bitmap_test() -> crate::Result<()> {
//         let column_len = ColumnCount::decode(&[26])?;
//         assert_eq!(column_len.columns, 26);
//
//         let types: Vec<TypeId> = vec![
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 2, 105, 100, 2, 105, 100, 12, 63, 0, 11, 0, 0,
//                 0, 3, 11, 66, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 50, 6, 102, 105,
//                 101, 108, 100, 50, 12, 224, 0, 120, 0, 0, 0, 253, 5, 64, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 51, 6, 102, 105,
//                 101, 108, 100, 51, 12, 224, 0, 252, 3, 0, 0, 253, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 52, 6, 102, 105,
//                 101, 108, 100, 52, 12, 63, 0, 4, 0, 0, 0, 1, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 53, 6, 102, 105,
//                 101, 108, 100, 53, 12, 63, 0, 19, 0, 0, 0, 7, 128, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 54, 6, 102, 105,
//                 101, 108, 100, 54, 12, 63, 0, 19, 0, 0, 0, 7, 128, 4, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 55, 6, 102, 105,
//                 101, 108, 100, 55, 12, 63, 0, 4, 0, 0, 0, 1, 1, 64, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 56, 6, 102, 105,
//                 101, 108, 100, 56, 12, 224, 0, 252, 255, 3, 0, 252, 16, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 6, 102, 105, 101, 108, 100, 57, 6, 102, 105,
//                 101, 108, 100, 57, 12, 63, 0, 4, 0, 0, 0, 1, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 48, 7, 102,
//                 105, 101, 108, 100, 49, 48, 12, 224, 0, 252, 3, 0, 0, 252, 16, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 49, 7, 102,
//                 105, 101, 108, 100, 49, 49, 12, 224, 0, 252, 3, 0, 0, 252, 16, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 50, 7, 102,
//                 105, 101, 108, 100, 49, 50, 12, 63, 0, 19, 0, 0, 0, 7, 129, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 51, 7, 102,
//                 105, 101, 108, 100, 49, 51, 12, 63, 0, 4, 0, 0, 0, 1, 0, 64, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 52, 7, 102,
//                 105, 101, 108, 100, 49, 52, 12, 63, 0, 11, 0, 0, 0, 3, 0, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 53, 7, 102,
//                 105, 101, 108, 100, 49, 53, 12, 63, 0, 11, 0, 0, 0, 3, 0, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 54, 7, 102,
//                 105, 101, 108, 100, 49, 54, 12, 63, 0, 4, 0, 0, 0, 1, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 55, 7, 102,
//                 105, 101, 108, 100, 49, 55, 12, 224, 0, 0, 1, 0, 0, 253, 0, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 56, 7, 102,
//                 105, 101, 108, 100, 49, 56, 12, 63, 0, 11, 0, 0, 0, 3, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 49, 57, 7, 102,
//                 105, 101, 108, 100, 49, 57, 12, 63, 0, 11, 0, 0, 0, 3, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 50, 48, 7, 102,
//                 105, 101, 108, 100, 50, 48, 12, 63, 0, 19, 0, 0, 0, 7, 128, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 50, 49, 7, 102,
//                 105, 101, 108, 100, 50, 49, 12, 63, 0, 19, 0, 0, 0, 7, 128, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 50, 50, 7, 102,
//                 105, 101, 108, 100, 50, 50, 12, 63, 0, 3, 0, 0, 0, 3, 0, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 50, 51, 7, 102,
//                 105, 101, 108, 100, 50, 51, 12, 63, 0, 6, 0, 0, 0, 3, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 50, 52, 7, 102,
//                 105, 101, 108, 100, 50, 52, 12, 63, 0, 6, 0, 0, 0, 3, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 50, 53, 7, 102,
//                 105, 101, 108, 100, 50, 53, 12, 63, 0, 20, 0, 0, 0, 8, 1, 0, 0, 0, 0,
//             ])?,
//             ColumnDefinition::decode(&[
//                 3, 100, 101, 102, 4, 115, 113, 108, 120, 8, 97, 99, 99, 111, 117, 110, 116, 115, 8,
//                 97, 99, 99, 111, 117, 110, 116, 115, 7, 102, 105, 101, 108, 100, 50, 54, 7, 102,
//                 105, 101, 108, 100, 50, 54, 12, 63, 0, 11, 0, 0, 0, 3, 0, 0, 0, 0, 0,
//             ])?,
//         ]
//         .into_iter()
//         .map(|def| def.type_id)
//         .collect();
//
//         EofPacket::decode(&[254, 0, 0, 34, 0])?;
//
//         Row::read(
//             &[
//                 0, 64, 90, 229, 0, 4, 0, 0, 0, 4, 114, 117, 115, 116, 0, 0, 7, 228, 7, 1, 16, 8,
//                 10, 17, 0, 0, 4, 208, 7, 1, 1, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//             ],
//             &types,
//             true,
//         )?;
//
//         EofPacket::decode(&[254, 0, 0, 34, 0])?;
//         Ok(())
//     }
// }
