use super::super::{
    deserialize::{DeContext, Deserialize},
    packets::{ok::OkPacket, err::ErrPacket, eof::EofPacket, column::ColumnPacket, column_def::ColumnDefPacket, result_row::ResultRow},
};
use bytes::Bytes;
use failure::Error;
use crate::protocol::server::Message;
use crate::protocol::types::Capabilities;
use crate::protocol::decode::Decoder;

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

        println!("length: {:?}", length);
        println!("seq_no: {:?}", seq_no);

        let eof_packet = if !(ctx.conn.capabilities & Capabilities::CLIENT_DEPRECATE_EOF).is_empty() {
            Some(EofPacket::deserialize(ctx)?)
        } else {
            None
        };

        println!("column_packet: {:?}", column_packet);

        for col in &columns {
            println!("col: {:?}", col);
        }

        println!("eof_packet: {:?}", eof_packet);

        ctx.columns = column_packet.columns.clone();

        // TODO: Deserialize all rows
        let rows = vec![ResultRow::deserialize(ctx)?];

        if (ctx.conn.capabilities & Capabilities::CLIENT_DEPRECATE_EOF).is_empty() {
            println!("eof_packet: {:?}", EofPacket::deserialize(ctx)?);
        } else {
            println!("ok_packet: {:?}", OkPacket::deserialize(ctx)?);
        }

        println!("rows: {:?}", rows);

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
    use super::*;
    use crate::{__bytes_builder, connection::Connection};
    use bytes::{BufMut, Bytes};
    use crate::protocol::packets::{ok::OkPacket, err::ErrPacket, eof::EofPacket, result_row::ResultRow};

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

        match conn.next().await? {
            Some(Message::OkPacket(_)) => {},
            Some(message @ Message::ErrPacket(_)) => {
                failure::bail!("Received an ErrPacket packet: {:?}", message);
            },
            Some(message) => {
                failure::bail!("Received an unexpected packet type: {:?}", message);
            }
            None => {
                failure::bail!("Did not receive a packet when one was expected");
            }
        }

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
