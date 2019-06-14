// Reference: https://mariadb.com/kb/en/library/connection

use byteorder::{ByteOrder, LittleEndian};
use failure::Error;
use std::iter::FromIterator;
use bytes::Bytes;

pub trait Deserialize: Sized {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    InitialHandshakePacket(InitialHandshakePacket),
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

impl Deserialize for InitialHandshakePacket {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error> {
        let mut index = 0;
        let protocol_version = buf[0] as u8;
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
