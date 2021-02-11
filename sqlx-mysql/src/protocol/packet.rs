use std::fmt::Debug;

use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::{Error, Result};

use crate::MySqlDatabaseError;

#[derive(Debug)]
pub(crate) struct Packet {
    pub(crate) bytes: Bytes,
}

impl Packet {
    pub(crate) fn is_error(&self) -> bool {
        // if the first byte of the payload is 0xFF and the payload is an ERR packet
        !self.bytes.is_empty() && self.bytes[0] == 0xff
    }

    #[inline]
    pub(crate) fn deserialize<'de, T>(self) -> Result<T>
    where
        T: Deserialize<'de> + Debug,
    {
        self.deserialize_with(())
    }

    #[inline]
    pub(crate) fn deserialize_with<'de, T, Cx: 'de>(self, context: Cx) -> Result<T>
    where
        T: Deserialize<'de, Cx> + Debug,
    {
        if self.is_error() {
            // if the first byte of the payload is 0xFF and the payload is an ERR packet
            return Err(Error::connect(MySqlDatabaseError(self.deserialize()?)));
        }

        let packet = T::deserialize_with(self.bytes, context)?;

        log::trace!("read  > {:?}", packet);

        Ok(packet)
    }
}
