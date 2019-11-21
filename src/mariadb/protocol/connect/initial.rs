use crate::{
    io::Buf,
    mariadb::{
        io::BufExt,
        protocol::{Capabilities, ServerStatusFlag},
    },
};
use byteorder::LittleEndian;
use std::io;

#[derive(Debug)]
pub struct InitialHandshakePacket {
    pub protocol_version: u8,
    pub server_version: String,
    pub server_status: ServerStatusFlag,
    pub server_default_collation: u8,
    pub connection_id: u32,
    pub scramble: Box<[u8]>,
    pub capabilities: Capabilities,
    pub auth_plugin_name: Option<String>,
}

impl InitialHandshakePacket {
    pub(crate) fn decode(mut buf: &[u8]) -> io::Result<Self> {
        let protocol_version = buf.get_u8()?;
        let server_version = buf.get_str_nul()?.to_owned();
        let connection_id = buf.get_u32::<LittleEndian>()?;
        let mut scramble = Vec::with_capacity(8);

        // scramble 1st part (authentication seed) : string<8>
        scramble.extend_from_slice(&buf[..8]);
        buf.advance(8);

        // reserved : string<1>
        buf.advance(1);

        // server capabilities (1st part) : int<2>
        let capabilities_1 = buf.get_u16::<LittleEndian>()?;
        let mut capabilities = Capabilities::from_bits_truncate(capabilities_1.into());

        // server default collation : int<1>
        let server_default_collation = buf.get_u8()?;

        // status flags : int<2>
        let server_status = buf.get_u16::<LittleEndian>()?;

        // server capabilities (2nd part) : int<2>
        let capabilities_2 = buf.get_u16::<LittleEndian>()?;
        capabilities |= Capabilities::from_bits_truncate(((capabilities_2 as u32) << 16).into());

        // if (server_capabilities & PLUGIN_AUTH)
        let plugin_data_length = if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            // plugin data length : int<1>
            buf.get_u8()?
        } else {
            // 0x00 : int<1>
            buf.advance(0);
            0
        };

        // filler : string<6>
        buf.advance(6);

        // if (server_capabilities & CLIENT_MYSQL)
        if capabilities.contains(Capabilities::CLIENT_MYSQL) {
            // filler : string<4>
            buf.advance(4);
        } else {
            // server capabilities 3rd part . MariaDB specific flags : int<4>
            let capabilities_3 = buf.get_u32::<LittleEndian>()?;
            capabilities |= Capabilities::from_bits_truncate((capabilities_2 as u128) << 32);
        }

        // if (server_capabilities & CLIENT_SECURE_CONNECTION)
        if capabilities.contains(Capabilities::SECURE_CONNECTION) {
            // scramble 2nd part . Length = max(12, plugin data length - 9) : string<N>
            let len = ((plugin_data_length as isize) - 9).max(12) as usize;
            scramble.extend_from_slice(&buf[..len]);
            buf.advance(len);

            // reserved byte : string<1>
            buf.advance(1);
        }

        // if (server_capabilities & PLUGIN_AUTH)
        let auth_plugin_name = if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            Some(buf.get_str_nul()?.to_owned())
        } else {
            None
        };

        Ok(Self {
            protocol_version,
            server_version,
            server_default_collation,
            server_status: ServerStatusFlag::from_bits_truncate(server_status),
            connection_id,
            scramble: scramble.into_boxed_slice(),
            capabilities,
            auth_plugin_name,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::__bytes_builder;

    #[test]
    fn it_decodes_initial_handshake_packet() -> io::Result<()> {
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

        let _message = InitialHandshakePacket::decode(&buf)?;

        Ok(())
    }
}
