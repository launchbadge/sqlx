// Reference: https://mariadb.com/kb/en/library/connection

use byteorder::{ByteOrder, LittleEndian};
use failure::{Error, err_msg};
use bytes::{Bytes, BytesMut};

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
    pub scramble2: Option<Bytes>,
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

        let length = (buf[0] + (buf[1]<<8) + (buf[2]<<16)) as u32;
        index += 3;

        if buf.len() != length as usize {
            return Err(err_msg("Lengths to do not match"));
        }

        let sequence_number = buf[index];
        index += 1;

        if sequence_number != 0 {
            return Err(err_msg("Squence Number of Initial Handshake Packet is not 0"));
        }

        let protocol_version = buf[index] as u8;
        index += 1;

        let null_index = memchr::memchr(b'\0', &buf[index..]).unwrap();
        let server_version = Bytes::from(buf[index..null_index].to_vec());
        index = null_index + 1;

        let connection_id = LittleEndian::read_u32(&buf);
        index += 4;

        let auth_seed = Bytes::from(buf[index..index + 8].to_vec());
        index += 8;

        // Skip reserved byte
        index += 1;

        let mut capabilities = Capabilities::from_bits(LittleEndian::read_u16(&buf[index..]).into()).unwrap();
        index += 2;

        let collation = buf[index];
        index += 1;

        let status = LittleEndian::read_u16(&buf[index..]);
        index += 2;

        capabilities |= Capabilities::from_bits(LittleEndian::read_u16(&buf[index..]).into()).unwrap();
        index += 2;

        let mut plugin_data_length = 0;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            plugin_data_length = buf[index] as u8;
        }
        index += 1;

        // Skip filler
        index += 6;

        if (capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            capabilities |= Capabilities::from_bits(LittleEndian::read_u32(&buf[index..]).into()).unwrap();
        }
        index += 4;

        let mut scramble2: Option<Bytes> = None;
        let mut auth_plugin_name: Option<Bytes> = None;
        if !(capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            let len = std::cmp::max(12, plugin_data_length - 9);
            scramble2 = Some(Bytes::from(buf[index..index + len as usize].to_vec()));
        } else {
            let null_index = memchr::memchr(b'\0', &buf[index..]).unwrap();
            auth_plugin_name = Some(Bytes::from(buf[index..null_index].to_vec()));
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
            scramble2,
            auth_plugin_name,
        })
    }
}

#[inline]
fn deserialize_int_lenenc(buf: &Vec<u8>, index: &usize) -> (Option<usize>, usize) {
    match buf[*index] {
        0xFB => (None, *index + 1),
        0xFC => (Some(LittleEndian::read_u16(&buf[*index + 1..]) as usize), *index + 2),
        0xFD => (Some((buf[*index + 1] + buf[*index + 2] << 8 + buf[*index + 3] << 16) as usize), *index + 3),
        0xFE => (Some(LittleEndian::read_u64(&buf[*index..]) as usize), *index + 8),
        0xFF => panic!("int<lenenc> unprocessable first byte 0xFF"),
        _ => (Some(buf[*index] as usize), *index + 1),
    }
}

#[inline]
fn deserialize_int_3(buf: &Vec<u8>, index: &usize) -> (u32, usize) {
    ((buf[*index] + buf[index + 1] << 8 + buf[*index + 2] << 16) as u32, index + 3)
}

#[inline]
fn deserialize_int_2(buf: &Vec<u8>, index: &usize) -> (u16, usize) {
    (LittleEndian::read_u16(&buf[*index..]), index + 2)
}

#[inline]
fn deserialize_int_1(buf: &Vec<u8>, index: &usize) -> (u8, usize) {
    (buf[*index], index + 1)
}

#[inline]
fn deserialize_string_lenenc(buf: &Vec<u8>, index: &usize) -> (Bytes, usize) {
    let (length, index) = deserialize_int_3(&buf, &index);
    (Bytes::from(&buf[index..index + length as usize]), index + length as usize)
}

#[inline]
fn deserialize_string_fix(buf: &Vec<u8>, index: &usize, length: usize) -> (Bytes, usize) {
    (Bytes::from(&buf[*index..index + length as usize]), index + length as usize)
}

#[inline]
fn deserialize_string_eof(buf: &Vec<u8>, index: &usize) -> (Bytes, usize) {
    (Bytes::from(&buf[*index..]), buf.len())
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

        Ok(ErrPacket::default())
    }
}
