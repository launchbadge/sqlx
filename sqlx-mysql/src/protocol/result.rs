use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::{Error, Result};

use super::{Capabilities, ErrPacket, OkPacket};
use crate::MySqlDatabaseError;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub(crate) enum ResultPacket {
    Ok(OkPacket),
    Err(ErrPacket),
}

impl ResultPacket {
    pub(crate) fn into_result(self) -> Result<OkPacket> {
        match self {
            Self::Ok(ok) => Ok(ok),
            Self::Err(err) => Err(Error::connect(MySqlDatabaseError(err))),
        }
    }
}

impl Deserialize<'_, Capabilities> for ResultPacket {
    fn deserialize_with(buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        Ok(if buf[0] == 0xff {
            Self::Err(ErrPacket::deserialize(buf)?)
        } else {
            Self::Ok(OkPacket::deserialize_with(buf, capabilities)?)
        })
    }
}
