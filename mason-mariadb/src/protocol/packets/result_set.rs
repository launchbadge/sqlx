use bytes::Bytes;
use failure::Error;

use crate::protocol::decode::Decoder;
use crate::protocol::server::Message;
use crate::protocol::types::Capabilities;

use super::super::{
    deserialize::{DeContext, Deserialize},
    packets::{column::ColumnPacket, column_def::ColumnDefPacket, eof::EofPacket, err::ErrPacket, ok::OkPacket, result_row::ResultRow},
};

#[derive(Debug, Default)]
pub struct ResultSet {
    pub length: u32,
    pub seq_no: u8,
    pub column_packet: ColumnPacket,
    pub columns: Vec<ColumnDefPacket>,
    pub rows: Vec<ResultRow>,
}

impl Deserialize for ResultSet {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let length = ctx.decoder.decode_length()?;
        let seq_no = ctx.decoder.decode_int_1();

        let column_packet = ColumnPacket::deserialize(ctx)?;

        let columns = if let Some(columns) = column_packet.columns {
            (0..columns)
                .map(|_| ColumnDefPacket::deserialize(ctx))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>()
        } else {
            Vec::new()
        };

        let eof_packet = if !(ctx.conn.capabilities & Capabilities::CLIENT_DEPRECATE_EOF).is_empty() {
            Some(EofPacket::deserialize(ctx)?)
        } else {
            None
        };

        ctx.columns = column_packet.columns.clone();

        let mut rows = Vec::new();

        loop {
            let packet_header = match ctx.decoder.peek_packet_header() {
                Ok(v) => v,
                Err(_) => break,
            };

            let tag = ctx.decoder.peek_tag();
            if tag == Some(&0xFE) && packet_header.length <= 0xFFFFFF {
                break;
            } else {
                let index = ctx.decoder.index;
                match ResultRow::deserialize(ctx) {
                    Ok(v) => rows.push(v),
                    Err(_) => {
                        ctx.decoder.index = index;
                        break;
                    },
                }
            }
        }

        if (ctx.conn.capabilities & Capabilities::CLIENT_DEPRECATE_EOF).is_empty() {
            EofPacket::deserialize(ctx)?;
        } else {
            OkPacket::deserialize(ctx)?;
        }

        Ok(ResultSet {
            length,
            seq_no,
            column_packet,
            columns,
            rows
        })
    }
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, Bytes};

    use crate::{__bytes_builder, connection::Connection};
    use crate::protocol::packets::{eof::EofPacket, err::ErrPacket, ok::OkPacket, result_row::ResultRow};

    use super::*;

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

        conn.select_db("test").await?;

        conn.query("SELECT * FROM users").await?;

        let buf = conn.stream.next_bytes().await?;
        let mut ctx = DeContext::new(&mut conn.context, &buf);

        ResultSet::deserialize(&mut ctx)?;

        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // ------------------- //
            // Column Count packet //
            // ------------------- //

            // length
            2u8, 0u8, 0u8,
            // seq_no
            2u8,
            // int<lenenc> Column count packet
            2u8, 0u8,

            // ------------------------ //
            // Column Definition packet //
            // ------------------------ //

            // length
            2u8, 0u8, 0u8,
            // seq_no
            2u8,
            // string<lenenc> catalog (always 'def')
            3u8, b"def",
            // string<lenenc> schema
            1u8, b'b',
            // string<lenenc> table alias
            1u8, b'c',
            // string<lenenc> table
            1u8, b'd',
            // string<lenenc> column alias
            1u8, b'e',
            // string<lenenc> column
            1u8, b'f',
            // int<lenenc> length of fixed fields (=0xC)
            0xFC_u8, 1u8, 1u8,
            // int<2> character set number
            1u8, 1u8,
            // int<4> max. column size
            1u8, 1u8, 1u8, 1u8,
            // int<1> Field types
            0u8,
            // int<2> Field detail flag
            0u8, 0u8,
            // int<1> decimals
            1u8,
            // int<2> - unused -
            0u8, 0u8,

            // ------------------------ //
            // Column Definition packet //
            // ------------------------ //

            // length
            2u8, 0u8, 0u8,
            // seq_no
            2u8,
            // string<lenenc> catalog (always 'def')
            3u8, b"def",
            // string<lenenc> schema
            1u8, b'b',
            // string<lenenc> table alias
            1u8, b'c',
            // string<lenenc> table
            1u8, b'd',
            // string<lenenc> column alias
            1u8, b'e',
            // string<lenenc> column
            1u8, b'f',
            // int<lenenc> length of fixed fields (=0xC)
            0xFC_u8, 1u8, 1u8,
            // int<2> character set number
            1u8, 1u8,
            // int<4> max. column size
            1u8, 1u8, 1u8, 1u8,
            // int<1> Field types
            0u8,
            // int<2> Field detail flag
            0u8, 0u8,
            // int<1> decimals
            1u8,
            // int<2> - unused -
            0u8, 0u8,

            // ---------- //
            // EOF Packet //
            // ---------- //

            // length
            1u8, 0u8, 0u8,
            // seq_no
            1u8,
            // int<1> 0xfe : EOF header
            0xFE_u8,
            // int<2> warning count
            0u8, 0u8,
            // int<2> server status
            1u8, 0u8,

            // ------------------- //
            // N Result Row Packet //
            // ------------------- //

            // string<lenenc> column data
            1u8, b'h',
            // string<lenenc> column data
            1u8, b'i',

            // ---------- //
            // EOF Packet //
            // ---------- //

            // length
            1u8, 0u8, 0u8,
            // seq_no
            1u8,
            // int<1> 0xfe : EOF header
            0xFE_u8,
            // int<2> warning count
            0u8, 0u8,
            // int<2> server status
            1u8, 0u8
        );

        Ok(())
    }
}
