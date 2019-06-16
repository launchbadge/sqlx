// Reference: https://mariadb.com/kb/en/library/connection

use crate::protocol::deserialize::*;
use byteorder::{ByteOrder, LittleEndian};
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
    pub server_status: u16,
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
            Capabilities::from_bits(deserialize_int_2(&buf, &mut index).into()).unwrap();

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
            capabilities |=
                Capabilities::from_bits(deserialize_int_4(&buf, &mut index).into()).unwrap();
        } else {
            // Skip filler
            index += 4;
        }

        let mut scramble: Option<Bytes> = None;
        let mut auth_plugin_name: Option<Bytes> = None;
        if !(capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            let len = std::cmp::max(12, plugin_data_length as usize - 9);
            scramble = Some(deserialize_string_fix(&buf, &mut index, len));
        } else {
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
        let mut index = 1;
        let affected_rows = deserialize_int_lenenc(&buf, &mut index);
        let last_insert_id = deserialize_int_lenenc(&buf, &mut index);
        let server_status = deserialize_int_2(&buf, &mut index);
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
        let mut index = 1;
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
