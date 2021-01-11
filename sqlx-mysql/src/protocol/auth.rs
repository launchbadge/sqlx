use std::fmt::Debug;

use bytes::Bytes;
use sqlx_core::io::{Deserialize, Serialize};
use sqlx_core::{Error, Result};

use crate::protocol::{AuthSwitch, Capabilities, MaybeCommand, OkPacket};
use crate::MySqlDatabaseError;

#[derive(Debug)]
pub(crate) enum Auth {
    Ok(OkPacket),
    MoreData(Bytes),
    Switch(AuthSwitch),
}

impl Deserialize<'_, Capabilities> for Auth {
    fn deserialize_with(buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        match buf[0] {
            0x00 => OkPacket::deserialize_with(buf, capabilities).map(Self::Ok),
            0x01 => Ok(Self::MoreData(buf.slice(1..))),
            0xfe => AuthSwitch::deserialize_with(buf, capabilities).map(Self::Switch),

            tag => Err(Error::connect(MySqlDatabaseError::new(
                2027,
                &format!(
                    "Malformed packet: Received 0x{:x} but expected one of: 0x0, 0x1, or 0xfe",
                    tag
                ),
            ))),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AuthResponse {
    pub(crate) data: Vec<u8>,
}

impl MaybeCommand for AuthResponse {}

impl Serialize<'_, Capabilities> for AuthResponse {
    fn serialize_with(&self, buf: &mut Vec<u8>, _context: Capabilities) -> Result<()> {
        buf.extend_from_slice(&self.data);

        Ok(())
    }
}
