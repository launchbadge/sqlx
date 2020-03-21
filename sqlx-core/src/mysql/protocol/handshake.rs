use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::protocol::{AuthPlugin, Capabilities, Status};
use crate::mysql::MySql;

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_connection_phase_packets_protocol_handshake_v10.html
// https://mariadb.com/kb/en/connection/#initial-handshake-packet
#[derive(Debug)]
pub(crate) struct Handshake {
    pub(crate) protocol_version: u8,
    pub(crate) server_version: Box<str>,
    pub(crate) connection_id: u32,
    pub(crate) server_capabilities: Capabilities,
    pub(crate) server_default_collation: u8,
    pub(crate) status: Status,
    pub(crate) auth_plugin: AuthPlugin,
    pub(crate) auth_plugin_data: Box<[u8]>,
}

impl Handshake {
    pub(crate) fn read(mut buf: &[u8]) -> crate::Result<MySql, Self>
    where
        Self: Sized,
    {
        let protocol_version = buf.get_u8()?;
        let server_version = buf.get_str_nul()?.into();
        let connection_id = buf.get_u32::<LittleEndian>()?;

        let mut scramble = Vec::with_capacity(8);

        // scramble first part : string<8>
        scramble.extend_from_slice(&buf[..8]);
        buf.advance(8);

        // reserved : string<1>
        buf.advance(1);

        // capability_flags_1 : int<2>
        let capabilities_1 = buf.get_u16::<LittleEndian>()?;
        let mut capabilities = Capabilities::from_bits_truncate(capabilities_1.into());

        // character_set : int<1>
        let char_set = buf.get_u8()?;

        // status_flags : int<2>
        let status = buf.get_u16::<LittleEndian>()?;
        let status = Status::from_bits_truncate(status);

        // capability_flags_2 : int<2>
        let capabilities_2 = buf.get_u16::<LittleEndian>()?;
        capabilities |= Capabilities::from_bits_truncate(((capabilities_2 as u32) << 16).into());

        let auth_plugin_data_len = if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            // plugin data length : int<1>
            buf.get_u8()?
        } else {
            // 0x00 : int<1>
            buf.advance(0);
            0
        };

        // reserved: string<6>
        buf.advance(6);

        if capabilities.contains(Capabilities::MYSQL) {
            // reserved: string<4>
            buf.advance(4);
        } else {
            // capability_flags_3 : int<4>
            let capabilities_3 = buf.get_u32::<LittleEndian>()?;
            capabilities |= Capabilities::from_bits_truncate((capabilities_3 as u64) << 32);
        }

        if capabilities.contains(Capabilities::SECURE_CONNECTION) {
            // scramble 2nd part : string<n> ( Length = max(12, plugin data length - 9) )
            let len = ((auth_plugin_data_len as isize) - 9).max(12) as usize;
            scramble.extend_from_slice(&buf[..len]);
            buf.advance(len);

            // reserved : string<1>
            buf.advance(1);
        }

        let auth_plugin = if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            AuthPlugin::from_opt_str(Some(buf.get_str_nul()?))?
        } else {
            AuthPlugin::from_opt_str(None)?
        };

        Ok(Self {
            protocol_version,
            server_capabilities: capabilities,
            server_version,
            server_default_collation: char_set,
            connection_id,
            auth_plugin_data: scramble.into_boxed_slice(),
            auth_plugin,
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthPlugin, Capabilities, Handshake, Status};
    use matches::assert_matches;

    const HANDSHAKE_MARIA_DB_10_4_7: &[u8] = b"\n5.5.5-10.4.7-MariaDB-1:10.4.7+maria~bionic\x00\x0b\x00\x00\x00t6L\\j\"dS\x00\xfe\xf7\x08\x02\x00\xff\x81\x15\x00\x00\x00\x00\x00\x00\x07\x00\x00\x00U14Oph9\"<H5n\x00mysql_native_password\x00";
    const HANDSHAKE_MYSQL_8_0_18: &[u8] = b"\n8.0.18\x00\x19\x00\x00\x00\x114aB0c\x06g\x00\xff\xff\xff\x02\x00\xff\xc7\x15\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00tL\x03s\x0f[4\rl4. \x00caching_sha2_password\x00";

    #[test]
    fn it_reads_handshake_mysql_8_0_18() {
        let mut p = Handshake::read(HANDSHAKE_MYSQL_8_0_18).unwrap();

        assert_eq!(p.protocol_version, 10);

        p.server_capabilities.toggle(
            Capabilities::MYSQL
                | Capabilities::FOUND_ROWS
                | Capabilities::LONG_FLAG
                | Capabilities::CONNECT_WITH_DB
                | Capabilities::NO_SCHEMA
                | Capabilities::COMPRESS
                | Capabilities::ODBC
                | Capabilities::LOCAL_FILES
                | Capabilities::IGNORE_SPACE
                | Capabilities::PROTOCOL_41
                | Capabilities::INTERACTIVE
                | Capabilities::SSL
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
                | Capabilities::MULTI_STATEMENTS
                | Capabilities::MULTI_RESULTS
                | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::CONNECT_ATTRS
                | Capabilities::PLUGIN_AUTH_LENENC_DATA
                | Capabilities::CAN_HANDLE_EXPIRED_PASSWORDS
                | Capabilities::SESSION_TRACK
                | Capabilities::DEPRECATE_EOF
                | Capabilities::ZSTD_COMPRESSION_ALGORITHM
                | Capabilities::SSL_VERIFY_SERVER_CERT
                | Capabilities::OPTIONAL_RESULTSET_METADATA
                | Capabilities::REMEMBER_OPTIONS,
        );

        assert!(p.server_capabilities.is_empty());

        assert_eq!(p.server_default_collation, 255);
        assert!(p.status.contains(Status::SERVER_STATUS_AUTOCOMMIT));
        assert_matches!(p.auth_plugin, AuthPlugin::CachingSha2Password);

        assert_eq!(
            &*p.auth_plugin_data,
            &[17, 52, 97, 66, 48, 99, 6, 103, 116, 76, 3, 115, 15, 91, 52, 13, 108, 52, 46, 32,]
        );
    }

    #[test]
    fn it_reads_handshake_mariadb_10_4_7() {
        let mut p = Handshake::read(HANDSHAKE_MARIA_DB_10_4_7).unwrap();

        assert_eq!(p.protocol_version, 10);

        assert_eq!(
            &*p.server_version,
            "5.5.5-10.4.7-MariaDB-1:10.4.7+maria~bionic"
        );

        p.server_capabilities.toggle(
            Capabilities::FOUND_ROWS
                | Capabilities::LONG_FLAG
                | Capabilities::CONNECT_WITH_DB
                | Capabilities::NO_SCHEMA
                | Capabilities::COMPRESS
                | Capabilities::ODBC
                | Capabilities::LOCAL_FILES
                | Capabilities::IGNORE_SPACE
                | Capabilities::PROTOCOL_41
                | Capabilities::INTERACTIVE
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
                | Capabilities::MULTI_STATEMENTS
                | Capabilities::MULTI_RESULTS
                | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::CONNECT_ATTRS
                | Capabilities::PLUGIN_AUTH_LENENC_DATA
                | Capabilities::CAN_HANDLE_EXPIRED_PASSWORDS
                | Capabilities::SESSION_TRACK
                | Capabilities::DEPRECATE_EOF
                | Capabilities::REMEMBER_OPTIONS,
        );

        assert!(p.server_capabilities.is_empty());

        assert_eq!(p.server_default_collation, 8);
        assert!(p.status.contains(Status::SERVER_STATUS_AUTOCOMMIT));
        assert_matches!(p.auth_plugin, AuthPlugin::MySqlNativePassword);

        assert_eq!(
            &*p.auth_plugin_data,
            &[
                116, 54, 76, 92, 106, 34, 100, 83, 85, 49, 52, 79, 112, 104, 57, 34, 60, 72, 53,
                110,
            ]
        );
    }
}
