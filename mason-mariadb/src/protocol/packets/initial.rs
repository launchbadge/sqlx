use super::super::{
    deserialize::{Deserialize, DeContext},
    types::{Capabilities, ServerStatusFlag},
};
use bytes::Bytes;
use failure::{err_msg, Error};

#[derive(Default, Debug)]
pub struct InitialHandshakePacket {
    pub length: u32,
    pub seq_no: u8,
    pub protocol_version: u8,
    pub server_version: Bytes,
    pub connection_id: u32,
    pub auth_seed: Bytes,
    pub capabilities: Capabilities,
    pub collation: u8,
    pub status: ServerStatusFlag,
    pub plugin_data_length: u8,
    pub scramble: Option<Bytes>,
    pub auth_plugin_name: Option<Bytes>,
}

impl Deserialize for InitialHandshakePacket {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

        if seq_no != 0 {
            return Err(err_msg("Sequence Number of Initial Handshake Packet is not 0"));
        }

        let protocol_version = decoder.decode_int_1();
        let server_version = decoder.decode_string_null()?;
        let connection_id = decoder.decode_int_4();
        let auth_seed = decoder.decode_string_fix(8);

        // Skip reserved byte
        decoder.skip_bytes(1);

        let mut capabilities = Capabilities::from_bits_truncate(decoder.decode_int_2().into());

        let collation = decoder.decode_int_1();
        let status = ServerStatusFlag::from_bits_truncate(decoder.decode_int_2().into());

        capabilities |=
            Capabilities::from_bits_truncate(((decoder.decode_int_2() as u32) << 16).into());

        let mut plugin_data_length = 0;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            plugin_data_length = decoder.decode_int_1();
        } else {
            // Skip reserve byte
            decoder.skip_bytes(1);
        }

        // Skip filler
        decoder.skip_bytes(6);

        if (capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            capabilities |=
                Capabilities::from_bits_truncate(((decoder.decode_int_4() as u128) << 32).into());
        } else {
            // Skip filler
            decoder.skip_bytes(4);
        }

        let mut scramble: Option<Bytes> = None;
        if !(capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            let len = std::cmp::max(12, plugin_data_length as usize - 9);
            scramble = Some(decoder.decode_string_fix(len as u32));
            // Skip reserve byte
            decoder.skip_bytes(1);
        }

        let mut auth_plugin_name: Option<Bytes> = None;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            auth_plugin_name = Some(decoder.decode_string_null()?);
        }

        ctx.conn.capabilities = capabilities;
        ctx.conn.last_seq_no = seq_no;

        Ok(InitialHandshakePacket {
            length,
            seq_no,
            protocol_version,
            server_version,
            connection_id,
            auth_seed,
            capabilities,
            collation,
            status,
            plugin_data_length,
            scramble,
            auth_plugin_name,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BytesMut;
    use mason_core::ConnectOptions;

    #[runtime::test]
    async fn it_decodes_initial_handshake_packet() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        }).await?;

        let buf = BytesMut::from(b"\
        n\0\0\
        \0\
        \n\
        5.5.5-10.4.6-MariaDB-1:10.4.6+maria~bionic\0\
        \x13\0\0\0\
        ?~~|vZAu\
        \0\
        \xfe\xf7\
        \x08\
        \x02\0\
        \xff\x81\
        \x15\
        \0\0\0\0\0\0\
        \x07\0\0\0\
        JQ8cihP4Q}Dx\
        \0\
        mysql_native_password\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
                .to_vec(),
        );

        let _message = InitialHandshakePacket::deserialize(&mut conn, &mut Decoder::new(&buf.freeze()))?;

        Ok(())
    }
}
