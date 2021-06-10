use std::fmt::Debug;

use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::protocol::{AuthSwitch, Capabilities, ResultPacket};
use crate::MySqlClientError;

#[derive(Debug)]
pub(crate) enum AuthResponse {
    End(ResultPacket),
    Command(u8, Bytes),
    Switch(AuthSwitch),
}

impl Deserialize<'_, Capabilities> for AuthResponse {
    fn deserialize_with(buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        match buf.get(0) {
            // OK or ERR -> end the auth cycle
            Some(0x00) | Some(0xff) => {
                ResultPacket::deserialize_with(buf, capabilities).map(Self::End)
            }

            // switch to another auth plugin
            Some(0xfe) => AuthSwitch::deserialize(buf).map(Self::Switch),

            // send a command to the active auth plugin
            Some(command) => Ok(Self::Command(*command, buf.slice(1..))),

            None => Err(MySqlClientError::EmptyPacket { context: "auth response" }.into()),
        }
    }
}
