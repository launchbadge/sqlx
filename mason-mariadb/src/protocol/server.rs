// Reference: https://mariadb.com/kb/en/library/connection

use byteorder::{ByteOrder, LittleEndian};
use failure::{Error, err_msg};
use bytes::{Bytes, BytesMut};
use crate::protocol::deserialize::*;

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
        let length = buf[0] + buf[1] << 8 + buf[2] << 16;
        let sequence_number = buf[3];
        Ok(None)
    }
}

impl Deserialize for InitialHandshakePacket {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error> {
        let mut index = 0;

        let (length, index) = deserialize_int_3(&buf, &index);

        if buf.len() != length as usize {
            return Err(err_msg("Lengths to do not match"));
        }

        let mut sequence_number = 0;
        (sequence_number, index) = deserialize_int_1(&buf, &index);

        if sequence_number != 0 {
            return Err(err_msg("Squence Number of Initial Handshake Packet is not 0"));
        }

        let (protocol_version, index) = deserialize_int_1(&buf, &index);
        let (server_version, index) = deserialize_string_null(&buf, &index);
        let (connection_id, index) = deserialize_int_4(&buf, &index);
        let (auth_seed, index) = deserialize_string_fix(&buf, &index, 8);

        // Skip reserved byte
        index += 1;

        let (cap, index) = deserialize_int_2(&buf, &index);
        let mut capabilities = Capabilities::from_bits(cap.into()).unwrap();

        let (collation, index) = deserialize_int_1(&buf, &index);

        let (status, index) = deserialize_int_2(&buf, &index);

        let (cap, index) = deserialize_int_2(&buf, &index);
        capabilities |= Capabilities::from_bits(cap.into()).unwrap();

        let mut plugin_data_length = 0;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            let (plugin, i) = deserialize_int_1(&buf, &index);
            plugin_data_length = plugin;
            index = i;
        } else {
            // Skip reserve byte
            index += 1;
        }

        // Skip filler
        index += 6;

        if (capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            let (cap, i) = deserialize_int_4(&buf, &index);
            capabilities |= Capabilities::from_bits(cap.into()).unwrap();
            index = i;
        } else {
            // Skip filler
            index += 4;
        }

        let mut scramble: Option<Bytes> = None;
        let mut auth_plugin_name: Option<Bytes> = None;
        if !(capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            let len = std::cmp::max(12, plugin_data_length as usize - 9);
            let (scram, i) = deserialize_string_fix(&buf, &index, len);
            scramble = Some(scram);
            index = i;
        } else {
            let (plugin, i) = deserialize_string_null(&buf, &index);
            let auth_plugin_name = Some(plugin);
            index = i;
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
        let (affected_rows, index) = deserialize_int_lenenc(&buf, &index);
        let (last_insert_id, index) = deserialize_int_lenenc(&buf, &index);
        let (server_status, index) = deserialize_int_2(&buf, &index);
        let (warning_count, index) = deserialize_int_2(&buf, &index);

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
        let (error_code, index) = deserialize_int_2(&buf, &index);

        let mut stage = None;
        let mut max_stage = None;
        let mut progress = None;
        let mut progress_info = None;

        let mut sql_state_marker = None;
        let mut sql_state = None;
        let mut error_message = None;

        // Progress Reporting
        if error_code == 0xFFFF {
            let (d_stage, index) = deserialize_int_1(&buf, &index);
            let (d_max_stage, index) = deserialize_int_1(&buf, &index);
            let (d_progress, index) = deserialize_int_3(&buf, &index);
            let (d_progress_info, index) = deserialize_string_lenenc(&buf, &index);
            stage = Some(d_stage);
            max_stage = Some(d_max_stage);
            progress = Some(d_progress);
            progress_info = Some(d_progress_info);


        } else {
            if buf[index] == b'#' {
                let (d_sql_state_marker, index) = deserialize_string_fix(&buf, &index, 1);
                let (d_sql_state, index) = deserialize_string_fix(&buf, &index, 5);
                let (d_error_message, index) = deserialize_string_eof(&buf, &index);
                sql_state_marker = Some(d_sql_state_marker);
                sql_state = Some(d_sql_state);
                error_message = Some(d_error_message);
            } else {
                let (d_error_message, index) = deserialize_string_eof(&buf, &index);
                error_message = Some(d_error_message);
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
