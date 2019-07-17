use super::super::deserialize::{Deserialize, DeContext};
use failure::Error;

#[derive(Default, Debug)]
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
    use super::*;
    use mason_core::ConnectOptions;

    #[runtime::test]
    async fn it_decodes_column_packet_0x_fb() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        }).await?;

        let buf = Bytes::from(b"\x01\0\0\x01\xFB".to_vec());
        let message = ColumnPacket::deserialize(&mut conn, &mut Decoder::new(&buf))?;

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
        }).await?;

        let buf = Bytes::from(b"\x04\0\0\x01\xFD\x01\x01\x01".to_vec());
        let message = ColumnPacket::deserialize(&mut conn, &mut Decoder::new(&buf))?;

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
        }).await?;

        let buf = Bytes::from(b"\x03\0\0\x01\xFC\x01\x01".to_vec());
        let message = ColumnPacket::deserialize(&mut conn, &mut Decoder::new(&buf))?;

        assert_ne!(message.columns, Some(0x0100));

        Ok(())
    }
}

