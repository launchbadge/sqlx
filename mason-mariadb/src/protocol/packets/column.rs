use failure::Error;

use super::super::deserialize::{DeContext, Deserialize};

#[derive(Default, Debug, Clone, Copy)]
// ColumnPacket doesn't have a packet header because
// it's nested inside a result set packet
pub struct ColumnPacket {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Option<usize>,
}

impl Deserialize for ColumnPacket {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

        let columns = decoder.decode_int_lenenc();

        Ok(ColumnPacket { length, seq_no, columns })
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use mason_core::ConnectOptions;

    use crate::{__bytes_builder, connection::Connection, protocol::decode::Decoder};

    use super::*;

    #[runtime::test]
    async fn it_decodes_column_packet_0x_fb() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<lenenc> tag code: None
        0xFB_u8
        );

        let message = ColumnPacket::deserialize(&mut DeContext::new(&mut conn.context, &buf))?;

        assert_eq!(message.columns, None);

        Ok(())
    }

    #[runtime::test]
    async fn it_decodes_column_packet_0x_fd() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<lenenc> tag code: Some(3 bytes)
        0xFD_u8,
        // value: 3 bytes
        0x01_u8, 0x01_u8, 0x01_u8
        );

        let message = ColumnPacket::deserialize(&mut DeContext::new(&mut conn.context, &buf))?;

        assert_eq!(message.columns, Some(0x010101));

        Ok(())
    }

    #[runtime::test]
    async fn it_fails_to_decode_column_packet_0x_fc() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<lenenc> tag code: Some(3 bytes)
        0xFC_u8,
        // value: 2 bytes
        0x01_u8, 0x01_u8
        );

        let message = ColumnPacket::deserialize(&mut DeContext::new(&mut conn.context, &buf))?;

        assert_ne!(message.columns, Some(0x0100));

        Ok(())
    }
}
