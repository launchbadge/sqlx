use crate::mariadb::{BufExt, Capabilities, Decode};
use byteorder::LittleEndian;
use std::{io, pin::Pin};

#[derive(Default, Debug)]
pub struct ResultRow<'a> {
    pub columns: Vec<&'a [u8]>,
}

impl<'a> Decode<'a> for ResultRow<'a> {
    fn decode(buf: &'a [u8], _: Capabilities) -> io::Result<Self> {
        // let buffer = Pin::new(buf.into());
        // let mut buf: &[u8] = &*buffer;

        // // FIXME: Where to put number of columns to decode?
        // let columns = Vec::new();
        // if let Some(num_columns) = Some(0) {
        //     for _ in 0..num_columns {
        //         columns.push(buf.get_byte_lenenc::<LittleEndian>()?);
        //     }
        // }

        Ok(ResultRow { columns: vec![] })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::__bytes_builder;

    #[test]
    fn it_decodes_result_row_packet() -> io::Result<()> {
        #[rustfmt::skip]
            let buf = __bytes_builder!(
            // int<3> length
            1u8, 0u8, 0u8,
            // int<1> seq_no
            1u8,
            // string<lenenc> column data
            1u8, b"s"
        );

        let _message = ResultRow::decode(&buf, Capabilities::CLIENT_PROTOCOL_41)?;

        Ok(())
    }
}
