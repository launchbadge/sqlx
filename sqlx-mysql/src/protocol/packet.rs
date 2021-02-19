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
        let packet = T::deserialize_with(self.bytes, context)?;

        log::trace!("read  > {:?}", packet);

        Ok(packet)
    }
}
