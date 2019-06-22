// Reference: https://mariadb.com/kb/en/library/connection

use crate::protocol::deserialize::*;
use bytes::{Bytes, BytesMut};
use failure::{err_msg, Error};

pub trait Deserialize: Sized {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    InitialHandshakePacket(InitialHandshakePacket),
    OkPacket(OkPacket),
    ErrPacket(ErrPacket),
}

bitflags! {
    pub struct Capabilities: u128 {
        const CLIENT_MYSQL = 1;
        const FOUND_ROWS = 2;
        const CONNECT_WITH_DB = 8;
        const COMPRESS = 32;
        const LOCAL_FILES = 128;
        const IGNORE_SPACE = 256;
        const CLIENT_PROTOCOL_41 = 1 << 9;
        const CLIENT_INTERACTIVE = 1 << 10;
        const SSL = 1 << 11;
        const TRANSACTIONS = 1 << 12;
        const SECURE_CONNECTION = 1 << 13;
        const MULTI_STATEMENTS = 1 << 16;
        const MULTI_RESULTS = 1 << 17;
        const PS_MULTI_RESULTS = 1 << 18;
        const PLUGIN_AUTH = 1 << 19;
        const CONNECT_ATTRS = 1 << 20;
        const PLUGIN_AUTH_LENENC_CLIENT_DATA = 1 << 21;
        const CLIENT_SESSION_TRACK = 1 << 23;
        const CLIENT_DEPRECATE_EOF = 1 << 24;
        const MARIA_DB_CLIENT_PROGRESS = 1 << 32;
        const MARIA_DB_CLIENT_COM_MULTI = 1 << 33;
        const MARIA_CLIENT_STMT_BULK_OPERATIONS = 1 << 34;
    }
}

bitflags! {
    pub struct FieldTypes: u8 {
        const MYSQL_TYPE_TINY = 1;
        const MYSQL_TYPE_SHORT = 2;
        const MYSQL_TYPE_LONG = 3;
        const MYSQL_TYPE_FLOAT = 4;
        const MYSQL_TYPE_DOUBLE = 5;
        const MYSQL_TYPE_NULL = 6;
        const MYSQL_TYPE_TIMESTAMP = 7;
        const MYSQL_TYPE_LONGLONG = 8;
        const MYSQL_TYPE_INT24 = 9;
        const MYSQL_TYPE_DATE = 10;
        const MYSQL_TYPE_TIME = 11;
        const MYSQL_TYPE_DATETIME = 12;
        const MYSQL_TYPE_YEAR = 13;
        const MYSQL_TYPE_NEWDATE = 14;
        const MYSQL_TYPE_VARCHAR = 15;
        const MYSQL_TYPE_BIT = 16;
        const MYSQL_TYPE_TIMESTAMP2 = 17;
        const MYSQL_TYPE_DATETIME2 = 18;
        const MYSQL_TYPE_TIME2 = 19;
        const MYSQL_TYPE_JSON = 245;
        const MYSQL_TYPE_NEWDECIMAL = 246;
        const MYSQL_TYPE_ENUM = 247;
        const MYSQL_TYPE_SET = 248;
        const MYSQL_TYPE_TINY_BLOB = 249;
        const MYSQL_TYPE_MEDIUM_BLOB = 250;
        const MYSQL_TYPE_LONG_BLOB = 251;
        const MYSQL_TYPE_BLOB = 252;
        const MYSQL_TYPE_VAR_STRING = 253;
        const MYSQL_TYPE_STRING = 254;
        const MYSQL_TYPE_GEOMETRY = 255;
    }
}

bitflags! {
    pub struct FieldDetailFlag: u16 {
        const NOT_NULL = 1;
        const PRIMARY_KEY = 2;
        const UNIQUE_KEY = 4;
        const MULTIPLE_KEY = 8;
        const BLOB = 16;
        const UNSIGNED = 32;
        const ZEROFILL_FLAG = 64;
        const BINARY_COLLATION = 128;
        const ENUM = 256;
        const AUTO_INCREMENT = 512;
        const TIMESTAMP = 1024;
        const SET = 2048;
        const NO_DEFAULT_VALUE_FLAG = 4096;
        const ON_UPDATE_NOW_FLAG = 8192;
        const NUM_FLAG = 32768;
    }
}

bitflags! {
    pub struct ServerStatusFlag: u16 {
        const SERVER_STATUS_IN_TRANS = 1;
        const SERVER_STATUS_AUTOCOMMIT = 2;
        const SERVER_MORE_RESULTS_EXISTS = 8;
        const SERVER_QUERY_NO_GOOD_INDEX_USED = 16;
        const SERVER_QUERY_NO_INDEX_USED = 32;
        const SERVER_STATUS_CURSOR_EXISTS = 64;
        const SERVER_STATUS_LAST_ROW_SENT = 128;
        const SERVER_STATUS_DB_DROPPED = 1 << 8;
        const SERVER_STATUS_NO_BACKSLASH_ESAPES = 1 << 9;
        const SERVER_STATUS_METADATA_CHANGED = 1 << 10;
        const SERVER_QUERY_WAS_SLOW = 1 << 11;
        const SERVER_PS_OUT_PARAMS = 1 << 12;
        const SERVER_STATUS_IN_TRANS_READONLY = 1 << 13;
        const SERVER_SESSION_STATE_CHANGED = 1 << 14;
    }
}

pub enum SessionChangeType {
    SessionTrackSystemVariables = 0,
    SessionTrackSchema = 1,
    SessionTrackStateChange = 2,
    SessionTrackGTIDS = 3,
    SessionTrackTransactionCharacteristics = 4,
    SessionTrackTransactionState = 5,
}

impl Default for Capabilities {
    fn default() -> Self {
        Capabilities::CLIENT_MYSQL
    }
}

impl Default for ServerStatusFlag {
    fn default() -> Self {
        ServerStatusFlag::SERVER_STATUS_IN_TRANS
    }
}

#[derive(Default, Debug)]
pub struct InitialHandshakePacket {
    pub length: u32,
    pub sequence_number: u8,
    pub protocol_version: u8,
    pub server_version: Bytes,
    pub connection_id: u32,
    pub auth_seed: Bytes,
    pub capabilities: Capabilities,
    pub collation: u8,
    pub status: u16,
    pub plugin_data_length: u8,
    pub scramble: Option<Bytes>,
    pub auth_plugin_name: Option<Bytes>,
}

#[derive(Default, Debug)]
pub struct OkPacket {
    pub affected_rows: Option<usize>,
    pub last_insert_id: Option<usize>,
    pub server_status: ServerStatusFlag,
    pub warning_count: u16,
    pub info: Bytes,
    pub session_state_info: Option<Bytes>,
    pub value: Option<Bytes>,
}

#[derive(Default, Debug)]
pub struct ErrPacket {
    pub error_code: u16,
    pub stage: Option<u8>,
    pub max_stage: Option<u8>,
    pub progress: Option<u32>,
    pub progress_info: Option<Bytes>,
    pub sql_state_marker: Option<Bytes>,
    pub sql_state: Option<Bytes>,
    pub error_message: Option<Bytes>,
}

impl Message {
    pub fn deserialize(buf: &mut BytesMut) -> Result<Option<Self>, Error> {
        // let length = deserialize_int_3(buf, &
        // let sequence_number = buf[3];
        Ok(None)
    }
    pub fn init(buf: &mut BytesMut) -> Result<Self, Error> {
        Ok(Message::InitialHandshakePacket(InitialHandshakePacket::deserialize(&mut buf.to_vec())?))
    }
}

impl Deserialize for InitialHandshakePacket {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error> {
        let mut index = 0;

        let length = deserialize_int_3(&buf, &mut index);

        if buf.len() != length as usize {
            return Err(err_msg("Lengths to do not match"));
        }

        let sequence_number = deserialize_int_1(&buf, &mut index);

        if sequence_number != 0 {
            return Err(err_msg("Squence Number of Initial Handshake Packet is not 0"));
        }

        let protocol_version = deserialize_int_1(&buf, &mut index);
        let server_version = deserialize_string_null(&buf, &mut index);
        let connection_id = deserialize_int_4(&buf, &mut index);
        let auth_seed = deserialize_string_fix(&buf, &mut index, 8);

        // Skip reserved byte
        index += 1;

        let mut capabilities =
            Capabilities::from_bits(deserialize_int_2(&buf, &mut index).into()).unwrap();

        let collation = deserialize_int_1(&buf, &mut index);
        let status = deserialize_int_2(&buf, &mut index);

        capabilities |=
            Capabilities::from_bits(((deserialize_int_2(&buf, &mut index) as u32) << 16).into())
                .unwrap();

        let mut plugin_data_length = 0;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            plugin_data_length = deserialize_int_1(&buf, &mut index);
        } else {
            // Skip reserve byte
            index += 1;
        }

        // Skip filler
        index += 6;

        if (capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            capabilities |= Capabilities::from_bits(
                ((deserialize_int_4(&buf, &mut index) as u128) << 32).into(),
            )
            .unwrap();
        } else {
            // Skip filler
            index += 4;
        }

        let mut scramble: Option<Bytes> = None;
        if !(capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            let len = std::cmp::max(12, plugin_data_length as usize - 9);
            scramble = Some(deserialize_string_fix(&buf, &mut index, len));
            // Skip reserve byte
            index += 1;
        }

        let mut auth_plugin_name: Option<Bytes> = None;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            auth_plugin_name = Some(deserialize_string_null(&buf, &mut index));
        }

        Ok(InitialHandshakePacket {
            length,
            sequence_number,
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

impl Deserialize for OkPacket {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error> {
        let mut index = 0;

        let length = deserialize_int_3(&buf, &mut index);

        if buf.len() != length as usize {
            return Err(err_msg("Lengths to do not match"));
        }

        let _sequence_number = deserialize_int_1(&buf, &mut index);

        let packet_header = deserialize_int_1(&buf, &mut index);
        if packet_header != 0 {
            panic!("Packet header is not 0 for OkPacket");
        }

        let affected_rows = deserialize_int_lenenc(&buf, &mut index);
        let last_insert_id = deserialize_int_lenenc(&buf, &mut index);
        let server_status = ServerStatusFlag::from_bits(deserialize_int_2(&buf, &mut index).into()).unwrap();
        let warning_count = deserialize_int_2(&buf, &mut index);

        // Assuming CLIENT_SESSION_TRACK is unsupported
        let session_state_info = None;
        let value = None;

        let info = Bytes::from(&buf[index..]);

        Ok(OkPacket {
            affected_rows,
            last_insert_id,
            server_status,
            warning_count,
            info,
            session_state_info,
            value,
        })
    }
}

impl Deserialize for ErrPacket {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error> {
        let mut index = 0;

        let length = deserialize_int_3(&buf, &mut index);

        if buf.len() != length as usize {
            return Err(err_msg("Lengths to do not match"));
        }

        let _sequence_number = deserialize_int_1(&buf, &mut index);

        let packet_header = deserialize_int_1(&buf, &mut index);
        if packet_header != 0xFF {
            panic!("Packet header is not 0xFF for ErrPacket");
        }

        let error_code = deserialize_int_2(&buf, &mut index);

        let mut stage = None;
        let mut max_stage = None;
        let mut progress = None;
        let mut progress_info = None;

        let mut sql_state_marker = None;
        let mut sql_state = None;
        let mut error_message = None;

        // Progress Reporting
        if error_code == 0xFFFF {
            stage = Some(deserialize_int_1(&buf, &mut index));
            max_stage = Some(deserialize_int_1(&buf, &mut index));
            progress = Some(deserialize_int_3(&buf, &mut index));
            progress_info = Some(deserialize_string_lenenc(&buf, &mut index));
        } else {
            if buf[index] == b'#' {
                sql_state_marker = Some(deserialize_string_fix(&buf, &mut index, 1));
                sql_state = Some(deserialize_string_fix(&buf, &mut index, 5));
                error_message = Some(deserialize_string_eof(&buf, &mut index));
            } else {
                error_message = Some(deserialize_string_eof(&buf, &mut index));
            }
        }

        Ok(ErrPacket {
            error_code,
            stage,
            max_stage,
            progress,
            progress_info,
            sql_state_marker,
            sql_state,
            error_message,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_decodes_capabilities() {
        let buf = b"\x00\x10".to_vec();
        let mut index = 0;
        Capabilities::from_bits(deserialize_int_2(&buf, &mut index).into()).unwrap();
    }

    #[test]
    fn it_decodes_initialhandshakepacket() -> Result<(), Error> {
        let mut buf = b"\
        \x54\x00\x00\
        \0\
        \x01\
        5.5.5-7\0\
        \x01\0\0\0\
        authseed\
        \0\
        \x00\x20\
        \0\
        \x00\x00\
        \x08\x00\
        \x0A\
        \0\0\0\0\0\0\
        \x01\x00\x00\x00\
        scrambled2nd\
        \0\
        authentication_plugin_name\0\
        "
        .to_vec();

        let message = InitialHandshakePacket::deserialize(&mut buf)?;
        assert_eq!(message.protocol_version, 1);
        assert_eq!(message.server_version, b"5.5.5-7".to_vec());
        assert_eq!(message.auth_seed, b"authseed".to_vec());
        assert_eq!(message.scramble, Some(Bytes::from(b"scrambled2nd".to_vec())));
        assert_eq!(
            message.auth_plugin_name,
            Some(Bytes::from(b"authentication_plugin_name".to_vec()))
        );
        assert!(!(message.capabilities & Capabilities::SECURE_CONNECTION).is_empty());
        assert!(!(message.capabilities & Capabilities::PLUGIN_AUTH).is_empty());
        assert!(!(message.capabilities & Capabilities::MARIA_DB_CLIENT_PROGRESS).is_empty());

        Ok(())
    }

    #[test]
    fn it_decodes_initialhandshakepacket_real() -> Result<(), Error> {
        let mut buf = b"\
        n\0\0\
        \0\
        \n\
        5.5.5-10.4.6-MariaDB-1:10.4\0".to_vec();

        let message = InitialHandshakePacket::deserialize(&mut buf)?;
//        assert_eq!(message.protocol_version, 1);
//        assert_eq!(message.server_version, b"5.5.5-7".to_vec());
//        assert_eq!(message.auth_seed, b"authseed".to_vec());
//        assert_eq!(message.scramble, Some(Bytes::from(b"scrambled2nd".to_vec())));
//        assert_eq!(
//            message.auth_plugin_name,
//            Some(Bytes::from(b"authentication_plugin_name".to_vec()))
//        );
//        assert!(!(message.capabilities & Capabilities::SECURE_CONNECTION).is_empty());
//        assert!(!(message.capabilities & Capabilities::PLUGIN_AUTH).is_empty());
//        assert!(!(message.capabilities & Capabilities::MARIA_DB_CLIENT_PROGRESS).is_empty());

        Ok(())
    }

    #[test]
    fn it_decodes_okpacket() -> Result<(), Error> {
        let mut buf = b"\
        \x0F\x00\x00\
        \x01\
        \x00\
        \xFB\
        \xFB\
        \x01\x01\
        \x00\x00\
        info\
        "
        .to_vec();

        let message = OkPacket::deserialize(&mut buf)?;

        assert_eq!(message.affected_rows, None);
        assert_eq!(message.last_insert_id, None);
        assert!(!(message.server_status & ServerStatusFlag::SERVER_STATUS_IN_TRANS).is_empty());
        assert_eq!(message.warning_count, 0);
        assert_eq!(message.info, b"info".to_vec());

        Ok(())
    }

    #[test]
    fn it_decodes_errpacket() -> Result<(), Error> {
        let mut buf = b"\
        \x0F\x00\x00\
        \x01\
        \xFF\
        \xEA\x03\
        #\
        HY000\
        NO\
        "
        .to_vec();

        let message = ErrPacket::deserialize(&mut buf)?;

        assert_eq!(message.error_code, 1002);
        assert_eq!(message.sql_state_marker, Some(Bytes::from(b"#".to_vec())));
        assert_eq!(message.sql_state, Some(Bytes::from(b"HY000".to_vec())));
        assert_eq!(message.error_message, Some(Bytes::from(b"NO".to_vec())));

        Ok(())
    }
}
