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
    pub column_packet: ColumnPacket,
    pub columns: Vec<ColumnDefPacket>,
    pub rows: Vec<ResultRow>,
}

impl Deserialize for ResultSet {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
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

        // TODO: Use byte string as input for test; this is a valid return from a mariadb.
        // Reference: b"\x01\0\0\x01\x04(\0\0\x02\x03def\x04test\x05users\x05users\x02id\x02id\x0c\x08\0\x80\0\0\0\xfd\x03@\0\0\04\0\0\x03\x03def\x04test\x05users\x05users\x08username\x08username\x0c\x08\0\xff\xff\0\0\xfc\x11\x10\0\0\04\0\0\x04\x03def\x04test\x05users\x05users\x08password\x08password\x0c\x08\0\xff\xff\0\0\xfc\x11\x10\0\0\0<\0\0\x05\x03def\x04test\x05users\x05users\x0caccess_level\x0caccess_level\x0c\x08\0\x07\0\0\0\xfe\x01\x11\0\0\0\x05\0\0\x06\xfe\0\0\"\0>\0\0\x07$044d3f34-af65-11e9-a2e5-0242ac110003\x04josh\x0bpassword123\x07regular4\0\0\x08$d83dd1c4-ada9-11e9-96bc-0242ac110003\x06daniel\x01f\x05admin\x05\0\0\t\xfe\0\0\"\0\0
        #[rustfmt::skip]
            let buf = __bytes_builder!(
        // ------------------- //
        // Column Count packet //
        // ------------------- //
        1u8, 0u8, 0u8,
        1u8,
        4u8,

        // ------------------------ //
        // Column Definition packet //
        // ------------------------ //
        40u8, 0u8, 0u8,
        2u8,
        3u8, b"def",
        4u8, b"test",
        5u8, b"users",
        5u8, b"users",
        2u8, b"id",
        2u8, b"id",
        0x0C_u8,
        8u8, 0u8,
        0x80_u8, 0u8, 0u8, 0u8,
        0xFD_u8,
        3u8, 64u8,
        0u8,
        0u8, 0u8,

        // ------------------------ //
        // Column Definition packet //
        // ------------------------ //
        52u8, 0u8, 0u8,
        3u8,
        3u8, b"def",
        4u8, b"test",
        5u8, b"users",
        5u8, b"users",
        8u8, b"username",
        8u8, b"username",
        0x0C_u8,
        8u8, 0u8,
        0xFF_u8, 0xFF_u8, 0u8, 0u8,
        0xFC_u8,
        0x11_u8, 0x10_u8,
        0u8,
        0u8, 0u8,

        // ------------------------ //
        // Column Definition packet //
        // ------------------------ //
        52u8, 0u8, 0u8,
        4u8,
        3u8, b"def",
        4u8, b"test",
        5u8, b"users",
        5u8, b"users",
        8u8, b"password",
        8u8, b"password",
        0x0C_u8,
        8u8, 0u8,
        0xFF_u8, 0xFF_u8, 0u8, 0u8,
        0xFC_u8,
        0x11_u8, 0x10_u8,
        0u8,
        0u8, 0u8,

        // ------------------------ //
        // Column Definition packet //
        // ------------------------ //
        60u8, 0u8, 0u8,
        5u8,
        3u8, b"def",
        4u8, b"test",
        5u8, b"users",
        5u8, b"users",
        12u8, b"access_level",
        12u8, b"access_level",
        12u8,
        8u8, 0u8,
        7u8, 0u8, 0u8, 0u8,
        0xFE_u8,
        1u8, 0x11_u8,
        0u8,
        0u8, 0u8,


        // ---------- //
        // EOF Packet //
        // ---------- //
        5u8, 0u8, 0u8,
        6u8,
        0xFE_u8,
        0u8, 0u8,
        34u8, 0u8,

        // ----------------- //
        // Result Row Packet //
        // ----------------- //
        62u8, 0u8, 0u8,
        7u8,
        32u8, b"044d3f34-af65-11e9-a2e5-0242ac110003",
        4u8, b"josh",
        11u8, b"password123",
        7u8, b"regular",

        // ----------------- //
        // Result Row Packet //
        // ----------------- //
        52u8, 0u8, 0u8,
        8u8,
        32u8, b"d83dd1c4-ada9-11e9-96bc-0242ac110003",
        6u8, b"daniel",
        1u8, b"f",
        5u8, b"admin",
        5u8, 0u8, 0u8,
        9u8,
        0xFE_u8,
        0u8, 0u8,
        34u8, 0u8
        );

        conn.select_db("test").await?;

        conn.query("SELECT * FROM users").await?;

        let buf = conn.stream.next_bytes().await?;
        println!("{:?}", buf);
        let mut ctx = DeContext::new(&mut conn.context, &buf);

        ResultSet::deserialize(&mut ctx)?;



            // ------------------------ //
            // Column Definition packet //
            // ------------------------ //

            // ---------- //
            // EOF Packet //
            // ---------- //

            // ------------------- //
            // N Result Row Packet //
            // ------------------- //

            // ---------- //
            // EOF Packet //
            // ---------- //

        Ok(())
    }
}
