// Reference: https://mariadb.com/kb/en/library/connection

use byteorder::{ByteOrder, LittleEndian};
use failure::Error;
use std::iter::FromIterator;

pub trait Deserialize: Sized {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    InitialHandshakePacket(InitialHandshakePacket),
}

pub enum Capabilities {
    ClientMysql = 1,
    FoundRows = 2,
    ConnectWithDb = 8,
    Compress = 32,
    LocalFiles = 128,
    IgnroeSpace = 256,
    ClientProtocol41 = 1 << 9,
    ClientInteractive = 1 << 10,
    SSL = 1 << 11,
    Transactions = 1 << 12,
    SecureConnection = 1 << 13,
    MultiStatements = 1 << 16,
    MultiResults = 1 << 17,
    PsMultiResults = 1 << 18,
    PluginAuth = 1 << 19,
    ConnectAttrs = 1 << 20,
    PluginAuthLenencClientData = 1 << 21,
    ClientSessionTrack = 1 << 23,
    ClientDeprecateEof = 1 << 24,
    MariaDbClientProgress = 1 << 32,
    MariaDbClientComMulti = 1 << 33,
    MariaClientStmtBulkOperations = 1 << 34,
}

#[derive(Default, Debug)]
pub struct InitialHandshakePacket {
    pub protocol_version: u8,
    pub server_version: String,
    pub connection_id: u32,
    pub auth_seed: String,
    pub reserved: u8,
    pub capabilities1: u16,
    pub collation: u8,
    pub status: u16,
    pub plugin_data_length: u8,
    pub scramble2: Option<String>,
    pub reserved2: Option<u8>,
    pub auth_plugin_name: Option<String>,
}

impl Deserialize for InitialHandshakePacket {
    fn deserialize(buf: &mut Vec<u8>) -> Result<Self, Error> {
        let mut index = 0;
        let mut null_index = 0;
        let protocol_version = buf[0] as u8;
        index += 1;

        // Find index of null character
        null_index = index;
        loop {
            if buf[null_index] == b'\0' {
                break;
            }
            null_index += 1;
        }
        let server_version = String::from_iter(
            buf[index..null_index]
                .iter()
                .map(|b| char::from(b.clone()))
                .collect::<Vec<char>>()
                .into_iter(),
        );
        // Script null character
        index = null_index + 1;

        let connection_id = LittleEndian::read_u32(&buf);

        // Increment by index by 4 bytes since we read a u32
        index += 4;

        let auth_seed = String::from_iter(
            buf[index..index + 8]
                .iter()
                .map(|b| char::from(b.clone()))
                .collect::<Vec<char>>()
                .into_iter(),
        );
        index += 8;

        // Skip reserved byte
        index += 1;

        let mut capabilities = LittleEndian::read_u16(&buf[index..]) as u32;
        index += 2;

        let collation = buf[index];
        index += 1;

        let status = LittleEndian::read_u16(&buf[index..]);
        index += 2;

        capabilities |= LittleEndian::read_u16(&buf[index..]) as u32;
        index += 2;

        let mut plugin_data_length = 0;
        if capabilities as u128 & Capabilities::PluginAuth as u128 > 0 {
            plugin_data_length = buf[index] as u8;
        }
        index += 1;

        // Skip filler
        index += 6;

        if capabilities as u128 & Capabilities::ClientMysql as u128 == 0 {
            capabilities |= LittleEndian::read_u32(&buf[index..]);
        }
        index += 4;

        let mut scramble2: Option<String> = None;
        let mut auth_plugin_name: Option<String> = None;
        if capabilities as u128 & Capabilities::SecureConnection as u128 > 0 {
            // TODO: scramble 2nd part. Length = max(12, plugin_data_length - 9)
            let len = std::cmp::max(12, plugin_data_length - 9);
            scramble2 = Some(String::from_iter(
                buf[index..index + len as usize]
                    .iter()
                    .map(|b| char::from(b.clone()))
                    .collect::<Vec<char>>()
                    .into_iter(),
            ));
            // Skip length characters + the reserved byte
            index += len as usize + 1;
        } else {
            // TODO: auth_plugin_name null temrinated string
            // Find index of null character
            null_index = index;
            loop {
                if buf[null_index] == b'\0' {
                    break;
                }
                null_index += 1;
            }
            auth_plugin_name = Some(String::from_iter(
                buf[index..null_index]
                    .iter()
                    .map(|b| char::from(b.clone()))
                    .collect::<Vec<char>>()
                    .into_iter(),
            ));
            // Script null character
            index = null_index + 1;
        }

        Ok(InitialHandshakePacket::default())
    }
}
