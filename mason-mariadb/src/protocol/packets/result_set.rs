use bytes::Bytes;
use failure::Error;
use crate::connection::Connection;

use super::super::{
    decode::Decoder,
    deserialize::Deserialize,
    packets::{column::ColumnPacket, column_def::ColumnDefPacket},
};

#[derive(Debug, Default)]
pub struct ResultSet {
    pub length: u32,
    pub seq_no: u8,
    pub column_packet: ColumnPacket,
    pub columns: Vec<ColumnDefPacket>,
    pub rows: Vec<Vec<Bytes>>,
}

impl Deserialize for ResultSet {
    fn deserialize(conn: &mut Connection, decoder: &mut Decoder) -> Result<Self, Error> {
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

        let column_packet = ColumnPacket::deserialize(conn, decoder)?;

        let columns = if let Some(columns) = column_packet.columns {
            (0..columns)
                .map(|_| ColumnDefPacket::deserialize(conn, decoder))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();

        for _ in 0.. {
            // if end of buffer stop
            if decoder.eof() {
                break;
            }

            // Decode each column as string<lenenc>
            rows.push(
                (0..column_packet.columns.unwrap_or(0))
                    .map(|_| decoder.decode_string_lenenc())
                    .collect::<Vec<Bytes>>(),
            )
        }

        Ok(ResultSet { length, seq_no, column_packet, columns, rows })
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use super::*;

    #[test]
    fn it_decodes_result_set_packet() -> Result<(), Error> {
        let buf = Bytes::from(b"\
        \0\0\0\x01\
        \x02\0\0\x02\xff\x02
        \x01\0\0a\
        \x01\0\0b\
        \x01\0\0c\
        \x01\0\0d\
        \x01\0\0e\
        \x01\0\0f\
        \xfc\x01\x01\
        \x01\x01\
        \x01\x01\x01\x01\
        \x00\
        \x00\x00\
        \x01\
        \0\0\
        \x01\0\0g\
        \x01\0\0h\
        \x01\0\0i\
        \x01\0\0j\
        \x01\0\0k\
        \x01\0\0l\
        \xfc\x01\x01\
        \x01\x01\
        \x01\x01\x01\x01\
        \x00\
        \x00\x00\
        \x01\
        \0\0
        ".to_vec());
        let message = ColumnDefPacket::deserialize(&mut Connection::mock(), &mut Decoder::new(&buf))?;

        Ok(())
    }
}
