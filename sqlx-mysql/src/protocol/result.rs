use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use super::{Capabilities, ErrPacket, OkPacket};
use crate::MySqlDatabaseError;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub(crate) enum ResultPacket<T = OkPacket>
where
    T: for<'de> Deserialize<'de, Capabilities>,
{
    Ok(T),
    Err(ErrPacket),
}

impl<T> ResultPacket<T>
where
    T: for<'de> Deserialize<'de, Capabilities>,
{
    pub(crate) fn into_result(self) -> Result<T> {
        match self {
            Self::Ok(ok) => Ok(ok),
            Self::Err(err) => Err(MySqlDatabaseError(err).into()),
        }
    }
}

impl<T> Deserialize<'_, Capabilities> for ResultPacket<T>
where
    T: for<'de> Deserialize<'de, Capabilities>,
{
    fn deserialize_with(buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        Ok(if buf[0] == 0xff {
            Self::Err(ErrPacket::deserialize(buf)?)
        } else {
            Self::Ok(T::deserialize_with(buf, capabilities)?)
        })
    }
}
