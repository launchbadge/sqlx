use crate::mariadb::{DeContext, Deserialize, Capabilities, ServerStatusFlag
};
use bytes::Bytes;
use failure::{err_msg, Error};

#[derive(Default, Debug)]
pub struct InitialHandshakePacket {
    pub length: u32,
    pub seq_no: u8,
    pub protocol_version: u8,
    pub server_version: Bytes,
    pub connection_id: i32,
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
        let seq_no = decoder.decode_int_u8();

        if seq_no != 0 {
            return Err(err_msg("Sequence Number of Initial Handshake Packet is not 0"));
        }

        let protocol_version = decoder.decode_int_u8();
        let server_version = decoder.decode_string_null()?;
        let connection_id = decoder.decode_int_i32();
        let auth_seed = decoder.decode_string_fix(8);

        // Skip reserved byte
        decoder.skip_bytes(1);

        let mut capabilities = Capabilities::from_bits_truncate(decoder.decode_int_u16().into());

        let collation = decoder.decode_int_u8();
        let status = ServerStatusFlag::from_bits_truncate(decoder.decode_int_u16().into());

        capabilities |=
            Capabilities::from_bits_truncate(((decoder.decode_int_i16() as u32) << 16).into());

        let mut plugin_data_length = 0;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            plugin_data_length = decoder.decode_int_u8();
        } else {
            // Skip reserve byte
            decoder.skip_bytes(1);
        }

        // Skip filler
        decoder.skip_bytes(6);

        if (capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            capabilities |=
                Capabilities::from_bits_truncate(((decoder.decode_int_u32() as u128) << 32).into());
        } else {
            // Skip filler
            decoder.skip_bytes(4);
        }

        let mut scramble: Option<Bytes> = None;
        if !(capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            let len = std::cmp::max(12, plugin_data_length as usize - 9);
            scramble = Some(decoder.decode_string_fix(len as usize));
            // Skip reserve byte
            decoder.skip_bytes(1);
        }

        let mut auth_plugin_name: Option<Bytes> = None;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            auth_plugin_name = Some(decoder.decode_string_null()?);
        }

        ctx.ctx.last_seq_no = seq_no;

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
    use crate::{__bytes_builder, ConnectOptions, mariadb::{ConnContext, Decoder}};
    use bytes::BytesMut;

    #[test]
    fn it_decodes_initial_handshake_packet() -> Result<(), Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // int<3> length
            1u8, 0u8, 0u8,
            // int<1> seq_no
            0u8,
            //int<1> protocol version
            10u8,
            //string<NUL> server version (MariaDB server version is by default prefixed by "5.5.5-")
            b"5.5.5-10.4.6-MariaDB-1:10.4.6+maria~bionic\0",
            //int<4> connection id
            13u8, 0u8, 0u8, 0u8,
            //string<8> scramble 1st part (authentication seed)
            b"?~~|vZAu",
            //string<1> reserved byte
            0u8,
            //int<2> server capabilities (1st part)
            0xFEu8, 0xF7u8,
            //int<1> server default collation
            8u8,
            //int<2> status flags
            2u8, 0u8,
            //int<2> server capabilities (2nd part)
            0xFF_u8, 0x81_u8,

            //if (server_capabilities & PLUGIN_AUTH)
            //  int<1> plugin data length
                15u8,
            //else
            //  int<1> 0x00

            //string<6> filler
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            //if (server_capabilities & CLIENT_MYSQL)
            //    string<4> filler
            //else
            //    int<4> server capabilities 3rd part . MariaDB specific flags /* MariaDB 10.2 or later */
                7u8, 0u8, 0u8, 0u8,
            //if (server_capabilities & CLIENT_SECURE_CONNECTION)
            //  string<n> scramble 2nd part . Length = max(12, plugin data length - 9)
                b"JQ8cihP4Q}Dx",
            //  string<1> reserved byte
                0u8,
            //if (server_capabilities & PLUGIN_AUTH)
            //  string<NUL> authentication plugin name
                b"mysql_native_password\0"
        );

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        let _message = InitialHandshakePacket::deserialize(&mut ctx)?;

        Ok(())
    }
}
