use bytes::buf::Chain;
use bytes::{Buf, Bytes};
use memchr::memchr;
use sqlx_core::io::{BufExt, Deserialize};
use sqlx_core::Result;

use crate::protocol::{Capabilities, Status};

// https://dev.mysql.com/doc/internals/en/connection-phase-packets.html#packet-Protocol::HandshakeV10
// https://mariadb.com/kb/en/connection/#initial-handshake-packet

#[derive(Debug)]
pub(crate) struct Handshake {
    // (0x0a) protocol version
    pub(crate) protocol_version: u8,

    // human-readable server version
    pub(crate) server_version: string::String<Bytes>,

    pub(crate) connection_id: u32,

    pub(crate) capabilities: Capabilities,
    pub(crate) status: Status,

    // default server character set
    pub(crate) charset: Option<u8>,

    pub(crate) auth_plugin_data: Chain<Bytes, Bytes>,

    // name of the auth_method that the auth_plugin_data belongs to
    pub(crate) auth_plugin_name: Option<string::String<Bytes>>,
}

impl Deserialize<'_, Capabilities> for Handshake {
    fn deserialize_with(mut buf: Bytes, _: Capabilities) -> Result<Self> {
        let protocol_version = buf.get_u8();

        // UNSAFE: server version is known to be ASCII
        #[allow(unsafe_code)]
        let server_version = unsafe { buf.get_str_nul_unchecked()? };

        let connection_id = buf.get_u32_le();

        // first 8 bytes of the auth-plugin data
        let auth_plugin_data_1 = buf.split_to(8);

        buf.advance(1); // filler [00]

        let mut capabilities = Capabilities::from_bits_truncate(buf.get_u16_le().into());

        // from this point on, all additional packet fields are **optional**
        // the packet payload can end at any time

        let charset = if buf.is_empty() { None } else { Some(buf.get_u8()) };

        let status = if buf.is_empty() {
            Status::empty()
        } else {
            Status::from_bits_truncate(buf.get_u16_le())
        };

        if !buf.is_empty() {
            // upper 2 bytes of the capabilities flags
            capabilities |= Capabilities::from_bits_truncate(u64::from(buf.get_u16_le()) << 16);
        }

        let auth_plugin_data_len = if capabilities.contains(Capabilities::PLUGIN_AUTH) {
            buf.get_u8()
        } else {
            // a single 0 byte, if present
            if !buf.is_empty() {
                buf.advance(1);
            }

            0
        };

        if buf.len() >= 10 {
            // reserved (10, 0 bytes)
            buf.advance(10);
        }

        let mut auth_plugin_data_2 = Bytes::new();
        let mut auth_plugin_name = None;

        if capabilities.contains(Capabilities::SECURE_CONNECTION) {
            let len = (if auth_plugin_data_len > 8 { auth_plugin_data_len - 8 } else { 0 }).max(13);

            auth_plugin_data_2 = buf.split_to(len as usize);

            if capabilities.contains(Capabilities::PLUGIN_AUTH) {
                // due to Bug#59453 the auth-plugin-name is missing the terminating NUL-char
                // in versions prior to 5.5.10 and 5.6.2

                // ref: https://bugs.mysql.com/bug.php?id=59453

                // read to NUL or read to the end if we can't find a NUL

                let auth_plugin_name_end =
                    memchr(b'\0', &buf).unwrap_or(buf.len());

                // UNSAFE: auth plugin names are known to be ASCII
                #[allow(unsafe_code)]
                let auth_plugin_name_ =
                    unsafe { Some(buf.get_str_unchecked(auth_plugin_name_end)) };

                auth_plugin_name = auth_plugin_name_;
            }
        }

        Ok(Self {
            protocol_version,
            server_version,
            connection_id,
            charset,
            capabilities,
            status,
            auth_plugin_data: auth_plugin_data_1.chain(auth_plugin_data_2),
            auth_plugin_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use bytes::Buf;
    use sqlx_core::io::Deserialize;

    use super::{Capabilities, Handshake, Status};

    const EMPTY: Capabilities = Capabilities::empty();

    #[test]
    fn handshake_mysql_8_0_18() {
        const HANDSHAKE_MYSQL_8_0_18: &[u8] = b"\n8.0.18\x00\x19\x00\x00\x00\x114aB0c\x06g\x00\xff\xff\xff\x02\x00\xff\xc7\x15\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00tL\x03s\x0f[4\rl4. \x00caching_sha2_password\x00";

        let mut h = Handshake::deserialize_with(HANDSHAKE_MYSQL_8_0_18.into(), EMPTY).unwrap();

        assert_eq!(h.protocol_version, 10);

        assert_eq!(
            h.capabilities,
            Capabilities::LONG_PASSWORD
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
                | Capabilities::DEPRECATE_EOF,
        );

        assert_eq!(h.charset, Some(255));
        assert_eq!(h.status, Status::AUTOCOMMIT);
        assert_eq!(h.auth_plugin_name.as_deref(), Some("caching_sha2_password"));

        assert_eq!(
            &*h.auth_plugin_data.copy_to_bytes(h.auth_plugin_data.remaining()),
            &[17, 52, 97, 66, 48, 99, 6, 103, 116, 76, 3, 115, 15, 91, 52, 13, 108, 52, 46, 32, 0]
        );
    }

    #[test]
    fn handshake_mariadb_10_4_7() {
        const HANDSHAKE_MARIA_DB_10_4_7: &[u8] = b"\n5.5.5-10.4.7-MariaDB-1:10.4.7+maria~bionic\x00\x0b\x00\x00\x00t6L\\j\"dS\x00\xfe\xf7\x08\x02\x00\xff\x81\x15\x00\x00\x00\x00\x00\x00\x07\x00\x00\x00U14Oph9\"<H5n\x00mysql_native_password\x00";

        let mut h = Handshake::deserialize_with(HANDSHAKE_MARIA_DB_10_4_7.into(), EMPTY).unwrap();

        assert_eq!(h.protocol_version, 10);
        assert_eq!(&*h.server_version, "5.5.5-10.4.7-MariaDB-1:10.4.7+maria~bionic");

        assert_eq!(
            h.capabilities,
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
        );

        assert_eq!(h.charset, Some(8));
        assert_eq!(h.status, Status::AUTOCOMMIT);
        assert_eq!(h.auth_plugin_name.as_deref(), Some("mysql_native_password"));

        assert_eq!(
            &*h.auth_plugin_data.copy_to_bytes(h.auth_plugin_data.remaining()),
            &[
                116, 54, 76, 92, 106, 34, 100, 83, 85, 49, 52, 79, 112, 104, 57, 34, 60, 72, 53,
                110, 0
            ]
        );
    }

    #[test]
    fn handshake_mariadb_10_5_8() {
        const HANDSHAKE_MARIA_DB_10_5_8: &[u8] = b"\n5.5.5-10.5.8-MariaDB-1:10.5.8+maria~focal\0\x07\0\0\0'PB949cf\0\xfe\xf7-\x02\0\xff\x81\x15\0\0\0\0\0\0\x0f\0\0\0UY>hr&`3{55H\0mysql_native_password\0";

        let mut h = Handshake::deserialize_with(HANDSHAKE_MARIA_DB_10_5_8.into(), EMPTY).unwrap();

        assert_eq!(h.protocol_version, 10);
        assert_eq!(&*h.server_version, "5.5.5-10.5.8-MariaDB-1:10.5.8+maria~focal");

        assert_eq!(
            h.capabilities,
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
        );

        assert_eq!(h.charset, Some(45));
        assert_eq!(h.status, Status::AUTOCOMMIT);
        assert_eq!(h.auth_plugin_name.as_deref(), Some("mysql_native_password"));

        assert_eq!(
            &*h.auth_plugin_data.copy_to_bytes(h.auth_plugin_data.remaining()),
            &[
                39, 80, 66, 57, 52, 57, 99, 102, 85, 89, 62, 104, 114, 38, 96, 51, 123, 53, 53, 72,
                0
            ]
        );
    }

    #[test]
    fn handshake_mysql_5_6_50() {
        const HANDSHAKE_MYSQL_5_6_50: &[u8] = b"\n5.6.50\0\x01\0\0\0-VLYZ:Pd\0\xff\xf7\x08\x02\0\x7f\x80\x15\0\0\0\0\0\0\0\0\0\0'2f+BL8nGV[G\0mysql_native_password\0";

        let mut h = Handshake::deserialize_with(HANDSHAKE_MYSQL_5_6_50.into(), EMPTY).unwrap();

        assert_eq!(h.protocol_version, 10);

        assert_eq!(&*h.server_version, "5.6.50");

        assert_eq!(
            h.capabilities,
            Capabilities::LONG_PASSWORD
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
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
                | Capabilities::MULTI_STATEMENTS
                | Capabilities::MULTI_RESULTS
                | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::CONNECT_ATTRS
                | Capabilities::PLUGIN_AUTH_LENENC_DATA
                | Capabilities::CAN_HANDLE_EXPIRED_PASSWORDS
        );

        assert_eq!(h.charset, Some(8));
        assert_eq!(h.status, Status::AUTOCOMMIT);
        assert_eq!(h.auth_plugin_name.as_deref(), Some("mysql_native_password"));

        assert_eq!(
            &*h.auth_plugin_data.copy_to_bytes(h.auth_plugin_data.remaining()),
            &[45, 86, 76, 89, 90, 58, 80, 100, 39, 50, 102, 43, 66, 76, 56, 110, 71, 86, 91, 71, 0]
        );
    }

    #[test]
    fn handshake_mysql_5_0_96() {
        const HANDSHAKE_MYSQL_5_0_96: &[u8] = b"\n5.0.96\0\x03\0\0\0bs=sNiGe\0,\xa2\x08\x02\0\0\0\0\0\0\0\0\0\0\0\0\0\0IzMP)yLLx;[9\0";

        let mut h = Handshake::deserialize_with(HANDSHAKE_MYSQL_5_0_96.into(), EMPTY).unwrap();

        assert_eq!(h.protocol_version, 10);
        assert_eq!(&*h.server_version, "5.0.96");

        assert_eq!(
            h.capabilities,
            Capabilities::LONG_FLAG
                | Capabilities::CONNECT_WITH_DB
                | Capabilities::COMPRESS
                | Capabilities::PROTOCOL_41
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
        );

        assert_eq!(h.charset, Some(8));
        assert_eq!(h.status, Status::AUTOCOMMIT);
        assert_eq!(h.auth_plugin_name, None);

        assert_eq!(
            &*h.auth_plugin_data.copy_to_bytes(h.auth_plugin_data.remaining()),
            &[
                98, 115, 61, 115, 78, 105, 71, 101, 73, 122, 77, 80, 41, 121, 76, 76, 120, 59, 91,
                57, 0
            ]
        );
    }

    #[test]
    fn handshake_mysql_5_1_73() {
        const HANDSHAKE_MYSQL_5_1_73: &[u8] = b"\n5.1.73\0\x01\0\0\0<fllZ\\Bs\0\xff\xf7\x08\x02\0\0\0\0\0\0\0\0\0\0\0\0\0\0<qEC_87JO/9q\0";

        let mut h = Handshake::deserialize_with(HANDSHAKE_MYSQL_5_1_73.into(), EMPTY).unwrap();

        assert_eq!(h.protocol_version, 10);
        assert_eq!(&*h.server_version, "5.1.73");

        assert_eq!(
            h.capabilities,
            Capabilities::LONG_PASSWORD
                | Capabilities::LONG_FLAG
                | Capabilities::FOUND_ROWS
                | Capabilities::CONNECT_WITH_DB
                | Capabilities::NO_SCHEMA
                | Capabilities::COMPRESS
                | Capabilities::ODBC
                | Capabilities::LOCAL_FILES
                | Capabilities::IGNORE_SPACE
                | Capabilities::INTERACTIVE
                | Capabilities::PROTOCOL_41
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
        );

        assert_eq!(h.charset, Some(8));
        assert_eq!(h.status, Status::AUTOCOMMIT);
        assert_eq!(h.auth_plugin_name, None);

        assert_eq!(
            &*h.auth_plugin_data.copy_to_bytes(h.auth_plugin_data.remaining()),
            &[
                60, 102, 108, 108, 90, 92, 66, 115, 60, 113, 69, 67, 95, 56, 55, 74, 79, 47, 57,
                113, 0
            ]
        );
    }

    #[test]
    fn handshake_mysql_5_5_14() {
        const HANDSHAKE_MYSQL_5_5_14: &[u8] = b"\n5.5.14\0\x01\0\0\0`o-/CEp'\0\xff\xf7\x08\x02\0\x0f\x80\x15\0\0\0\0\0\0\0\0\0\0kf@J5j6nJfAP\0mysql_native_password\0";

        let mut h = Handshake::deserialize_with(HANDSHAKE_MYSQL_5_5_14.into(), EMPTY).unwrap();

        assert_eq!(h.protocol_version, 10);
        assert_eq!(&*h.server_version, "5.5.14");

        assert_eq!(
            h.capabilities,
            Capabilities::LONG_PASSWORD
                | Capabilities::LONG_FLAG
                | Capabilities::FOUND_ROWS
                | Capabilities::CONNECT_WITH_DB
                | Capabilities::NO_SCHEMA
                | Capabilities::COMPRESS
                | Capabilities::ODBC
                | Capabilities::LOCAL_FILES
                | Capabilities::MULTI_STATEMENTS
                | Capabilities::MULTI_RESULTS
                | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::IGNORE_SPACE
                | Capabilities::INTERACTIVE
                | Capabilities::PROTOCOL_41
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
        );

        assert_eq!(h.charset, Some(8));
        assert_eq!(h.status, Status::AUTOCOMMIT);
        assert_eq!(h.auth_plugin_name.as_deref(), Some("mysql_native_password"));

        assert_eq!(
            &*h.auth_plugin_data.copy_to_bytes(h.auth_plugin_data.remaining()),
            &[
                96, 111, 45, 47, 67, 69, 112, 39, 107, 102, 64, 74, 53, 106, 54, 110, 74, 102, 65,
                80, 0
            ]
        );
    }
}
