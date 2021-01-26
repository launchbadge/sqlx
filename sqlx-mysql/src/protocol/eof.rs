use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::protocol::{Capabilities, Status};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub(crate) struct EofPacket {
    pub(crate) status: Status,
    pub(crate) warnings: u16,
}

impl Deserialize<'_, Capabilities> for EofPacket {
    fn deserialize_with(mut buf: Bytes, capabilities: Capabilities) -> Result<Self> {
        let tag = buf.get_u8();
        debug_assert_eq!(tag, 0xfe);

        let status =
            if capabilities.intersects(Capabilities::PROTOCOL_41 | Capabilities::TRANSACTIONS) {
                Status::from_bits_truncate(buf.get_u16_le())
            } else {
                Status::empty()
            };

        let warnings =
            if capabilities.contains(Capabilities::PROTOCOL_41) { buf.get_u16_le() } else { 0 };

        Ok(Self { status, warnings })
    }
}
