use std::fmt::Debug;

use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::protocol::{AuthSwitch, Capabilities, ResultPacket};
use crate::MySqlDatabaseError;

#[derive(Debug)]
pub(crate) enum AuthResponse {
    End(ResultPacket),
    MoreData(Bytes),
    Switch(AuthSwitch),
}

impl Deserialize<'_, Capabilities> for AuthResponse {
    fn deserialize_with(buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        match buf.get(0) {
            Some(0x00) => ResultPacket::deserialize_with(buf, capabilities).map(Self::End),
            Some(0x01) => Ok(Self::MoreData(buf.slice(1..))),
            Some(0xfe) => AuthSwitch::deserialize(buf).map(Self::Switch),

            Some(tag) => Err(MySqlDatabaseError::malformed_packet(&format!(
                "Received 0x{:x} but expected one of: 0x0 (OK), 0x1 (MORE DATA), or 0xfe (SWITCH) for auth response",
                tag
            )).into()),

            None => Err(MySqlDatabaseError::malformed_packet(
                "Received no bytes for auth response",
            ).into()),
        }
    }
}
