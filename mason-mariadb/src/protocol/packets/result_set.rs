use super::super::{
    deserialize::{DeContext, Deserialize},
    packets::{column::ColumnPacket, column_def::ColumnDefPacket},
};
use bytes::Bytes;
use failure::Error;

#[derive(Debug, Default)]
pub struct ResultSet {
    pub length: u32,
    pub seq_no: u8,
    pub column_packet: ColumnPacket,
    pub columns: Vec<ColumnDefPacket>,
    pub rows: Vec<Vec<Bytes>>,
}

impl Deserialize for ResultSet {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let length = ctx.decoder.decode_length()?;
        let seq_no = ctx.decoder.decode_int_1();

        let column_packet = ColumnPacket::deserialize(ctx)?;

        match ctx.decoder.decode_int_1() {
            // 0x00 -> PACKET_OK
            0x00 => {}

            // 0xFF -> PACKET_ERR
            0xFF => {}

            _ => {
                panic!("Didn't receive 0x00 nor 0xFF");
            }
        }

        let columns = if let Some(columns) = column_packet.columns {
            (0..columns)
                .map(|_| ColumnDefPacket::deserialize(ctx))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();

        loop {
            // if end of buffer stop
            if ctx.decoder.eof() {
                break;
            }

            let columns = if let Some(columns) = column_packet.columns {
                (0..columns).map(|_| ctx.decoder.decode_string_lenenc()).collect::<Vec<Bytes>>()
            } else {
                Vec::new()
            };
        }

        Ok(ResultSet { length, seq_no, column_packet, columns, rows })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{__bytes_builder, connection::Connection};
    use bytes::{BufMut, Bytes};

    #[runtime::test]
    async fn it_decodes_result_set_packet() -> Result<(), Error> {
        let mut conn = Connection::establish(mason_core::ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        //        conn.query("SELECT * FROM users");

        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // ------------------- //
            // Column Count packet //
            // ------------------- //

            // length
            0x02_u8, 0x0_u8, 0x0_u8,
            // seq_no
            0x02_u8,
            // int<lenenc> Column count packet
            0x02_u8, 0x00_u8,

            // ------------------------ //
            // Column Definition packet //
            // ------------------------ //

            // length
            0x02_u8, 0x0_u8, 0x0_u8,
            // seq_no
            0x02_u8,
            // string<lenenc> catalog (always 'def')
            0x03_u8, 0x0_u8, 0x0_u8, b"def",
            // string<lenenc> schema
            0x01_u8, 0x0_u8, 0x0_u8, b'b',
            // string<lenenc> table alias
            0x01_u8, 0x0_u8, 0x0_u8, b'c',
            // string<lenenc> table
            0x01_u8, 0x0_u8, 0x0_u8, b'd',
            // string<lenenc> column alias
            0x01_u8, 0x0_u8, 0x0_u8, b'e',
            // string<lenenc> column
            0x01_u8, 0x0_u8, 0x0_u8, b'f',
            // int<lenenc> length of fixed fields (=0xC)
            0xfc_u8, 0x01_u8, 0x01_u8,
            // int<2> character set number
            0x01_u8, 0x01_u8,
            // int<4> max. column size
            0x01_u8, 0x01_u8, 0x01_u8, 0x01_u8,
            // int<1> Field types
            0x00_u8,
            // int<2> Field detail flag
            0x00_u8, 0x00_u8,
            // int<1> decimals
            0x01_u8,
            // int<2> - unused -
            0x0_u8, 0x0_u8,

            // ------------------------ //
            // Column Definition packet //
            // ------------------------ //

            // length
            0x02_u8, 0x0_u8, 0x0_u8,
            // seq_no
            0x02_u8,
            // string<lenenc> catalog (always 'def')
            0x03_u8, 0x0_u8, 0x0_u8, b"def",
            // string<lenenc> schema
            0x01_u8, 0x0_u8, 0x0_u8, b'b',
            // string<lenenc> table alias
            0x01_u8, 0x0_u8, 0x0_u8, b'c',
            // string<lenenc> table
            0x01_u8, 0x0_u8, 0x0_u8, b'd',
            // string<lenenc> column alias
            0x01_u8, 0x0_u8, 0x0_u8, b'e',
            // string<lenenc> column
            0x01_u8, 0x0_u8, 0x0_u8, b'f',
            // int<lenenc> length of fixed fields (=0xC)
            0xfc_u8, 0x01_u8, 0x01_u8,
            // int<2> character set number
            0x01_u8, 0x01_u8,
            // int<4> max. column size
            0x01_u8, 0x01_u8, 0x01_u8, 0x01_u8,
            // int<1> Field types
            0x00_u8,
             // int<2> Field detail flag
            0x00_u8, 0x00_u8,
            // int<1> decimals
            0x01_u8,
            // int<2> - unused -
            0x0_u8, 0x00_u8,

            // ---------- //
            // EOF Packet //
            // ---------- //

            // length
            0x02_u8, 0x0_u8, 0x0_u8,
            // seq_no
            0x02_u8,
            // int<1> 0xfe : EOF header
            0xfe_u8,
            // int<2> warning count
            0x0_u8, 0x0_u8,
            // int<2> server status
            0x01_u8, 0x00_u8,

            // ------------------- //
            // N Result Row Packet //
            // ------------------- //

            // string<lenenc> column data
            0x01_u8, 0x0_u8, 0x0_u8, b'h',
            // string<lenenc> column data
            0x01_u8, 0x0_u8, 0x0_u8, b'i',

            // ---------- //
            // EOF Packet //
            // ---------- //

            // length
            0x02_u8, 0x0_u8, 0x0_u8,
            // seq_no
            0x02_u8,
            // int<1> 0xfe : EOF header
            0xfe_u8,
            // int<2> warning count
            0x0_u8, 0x0_u8,
            // int<2> server status
            0x01_u8, 0x00_u8
        );

        Ok(())
    }
}
