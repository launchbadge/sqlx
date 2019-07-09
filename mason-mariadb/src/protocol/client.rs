// Reference: https://mariadb.com/kb/en/library/connection
// Packets: https://mariadb.com/kb/en/library/0-packet

// TODO: Handle lengths which are greater than 3 bytes
// Either break the backet into several smaller ones, or
// return error
// TODO: Handle different Capabilities for server and client
// TODO: Handle when capability is set, but field is None

use super::server::Capabilities;
use crate::protocol::encode::*;
use bytes::{Bytes, BytesMut};
use failure::Error;

pub trait Serialize {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error>;
}

pub enum TextProtocol {
    ComChangeUser = 0x11,
    ComDebug = 0x0D,
    ComInitDb = 0x02,
    ComPing = 0x0e,
    ComProcessKill = 0xC,
    ComQuery = 0x03,
    ComQuit = 0x01,
    ComResetConnection = 0x1F,
    ComSetOption = 0x1B,
    ComShutdown = 0x0A,
    ComSleep = 0x00,
    ComStatistics = 0x09,
}

#[derive(Clone, Copy)]
pub enum SetOptionOptions {
    MySqlOptionMultiStatementsOn = 0x00,
    MySqlOptionMultiStatementsOff = 0x01,
}

#[derive(Clone, Copy)]
pub enum ShutdownOptions {
    ShutdownDefault = 0x00,
}

impl Into<u8> for TextProtocol {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Into<u16> for SetOptionOptions {
    fn into(self) -> u16 {
        self as u16
    }
}

impl Into<u8> for ShutdownOptions {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug)]
pub struct SSLRequestPacket {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
}

#[derive(Default, Debug)]
pub struct HandshakeResponsePacket {
    pub capabilities: Capabilities,
    pub max_packet_size: u32,
    pub collation: u8,
    pub extended_capabilities: Option<Capabilities>,
    pub username: Bytes,
    pub auth_data: Option<Bytes>,
    pub auth_response_len: Option<u8>,
    pub auth_response: Option<Bytes>,
    pub database: Option<Bytes>,
    pub auth_plugin_name: Option<Bytes>,
    pub conn_attr_len: Option<usize>,
    pub conn_attr: Option<Vec<(Bytes, Bytes)>>,
}

pub struct ComQuit();
pub struct ComDebug();
pub struct ComPing();
pub struct ComResetConnection();
pub struct ComStatistics();
pub struct ComSleep();

pub struct ComInitDb {
    pub schema_name: Bytes,
}

pub struct ComProcessKill {
    pub process_id: u32,
}

pub struct ComQuery {
    pub sql_statement: Bytes,
}

pub struct ComSetOption {
    pub option: SetOptionOptions,
}

pub struct ComShutdown {
    pub option: ShutdownOptions,
}

#[derive(Default, Debug)]
pub struct AuthenticationSwitchRequestPacket {
    pub auth_plugin_name: Bytes,
    pub auth_plugin_data: Bytes,
}

impl Serialize for ComQuit {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComQuit.into());

        Ok(())
    }
}

impl Serialize for ComInitDb {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComInitDb.into());
        encode_string_null(buf, &self.schema_name);

        Ok(())
    }
}

impl Serialize for ComDebug {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComDebug.into());

        Ok(())
    }
}

impl Serialize for ComPing {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComPing.into());

        Ok(())
    }
}

impl Serialize for ComProcessKill {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComProcessKill.into());
        encode_int_4(buf, self.process_id);

        Ok(())
    }
}

impl Serialize for ComQuery {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComQuery.into());
        encode_string_eof(buf, &self.sql_statement);

        Ok(())
    }
}

impl Serialize for ComResetConnection {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComResetConnection.into());

        Ok(())
    }
}

impl Serialize for ComSetOption {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComSetOption.into());
        encode_int_2(buf, self.option.into());

        Ok(())
    }
}

impl Serialize for ComShutdown {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComShutdown.into());
        encode_int_1(buf, self.option.into());

        Ok(())
    }
}

impl Serialize for ComSleep {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComSleep.into());

        Ok(())
    }
}

impl Serialize for ComStatistics {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComStatistics.into());

        Ok(())
    }
}

impl Serialize for SSLRequestPacket {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_4(buf, self.capabilities.bits() as u32);
        encode_int_4(buf, self.max_packet_size);
        encode_int_1(buf, self.collation);

        // Filler
        encode_byte_fix(buf, &Bytes::from_static(&[0u8; 19]), 19);

        if !(*server_capabilities & Capabilities::CLIENT_MYSQL).is_empty()
            && !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
        {
            if let Some(capabilities) = self.extended_capabilities {
                encode_int_4(buf, capabilities.bits() as u32);
            }
        } else {
            encode_byte_fix(buf, &Bytes::from_static(&[0u8; 4]), 4);
        }

        Ok(())
    }
}

impl Serialize for HandshakeResponsePacket {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_4(buf, self.capabilities.bits() as u32);
        encode_int_4(buf, self.max_packet_size);
        encode_int_1(buf, self.collation);

        // Filler
        encode_byte_fix(buf, &Bytes::from_static(&[0u8; 19]), 19);

        if !(*server_capabilities & Capabilities::CLIENT_MYSQL).is_empty()
            && !(self.capabilities & Capabilities::CLIENT_MYSQL).is_empty()
        {
            if let Some(capabilities) = self.extended_capabilities {
                encode_int_4(buf, capabilities.bits() as u32);
            }
        } else {
            encode_byte_fix(buf, &Bytes::from_static(&[0u8; 4]), 4);
        }

        encode_string_null(buf, &self.username);

        if !(*server_capabilities & Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA).is_empty() {
            if let Some(auth_data) = &self.auth_data {
                encode_string_lenenc(buf, &auth_data);
            }
        } else if !(*server_capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            if let Some(auth_response) = &self.auth_response {
                encode_int_1(buf, self.auth_response_len.unwrap());
                encode_string_fix(buf, &auth_response, self.auth_response_len.unwrap() as usize);
            }
        } else {
            encode_int_1(buf, 0);
        }

        if !(*server_capabilities & Capabilities::CONNECT_WITH_DB).is_empty() {
            if let Some(database) = &self.database {
                // string<NUL>
                encode_string_null(buf, &database);
            }
        }

        if !(*server_capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            if let Some(auth_plugin_name) = &self.auth_plugin_name {
                // string<NUL>
                encode_string_null(buf, &auth_plugin_name);
            }
        }

        if !(*server_capabilities & Capabilities::CONNECT_ATTRS).is_empty() {
            if let (Some(conn_attr_len), Some(conn_attr)) = (&self.conn_attr_len, &self.conn_attr) {
                // int<lenenc>
                encode_int_lenenc(buf, Some(conn_attr_len));

                // Loop
                for (key, value) in conn_attr {
                    encode_string_lenenc(buf, &key);
                    encode_string_lenenc(buf, &value);
                }
            }
        }

        Ok(())
    }
}

impl Serialize for AuthenticationSwitchRequestPacket {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, 0xFE);
        encode_string_null(buf, &self.auth_plugin_name);
        encode_byte_eof(buf, &self.auth_plugin_data);

        Ok(())
    }
}
