use bytes::{buf::Chain, Buf, Bytes};
use sqlx_core::io::{BufExt, Deserialize};
use sqlx_core::Result;

use super::Capabilities;
use crate::protocol::AuthPlugin;

// https://dev.mysql.com/doc/internals/en/authentication-method-change.html
// https://dev.mysql.com/doc/internals/en/connection-phase-packets.html#packet-Protocol::AuthSwitchRequest

#[derive(Debug)]
pub(crate) struct AuthSwitch {
    pub(crate) plugin: Box<dyn AuthPlugin>,
    pub(crate) plugin_data: Chain<Bytes, Bytes>,
}

impl Deserialize<'_, Capabilities> for AuthSwitch {
    fn deserialize_with(mut buf: Bytes, _capabilities: Capabilities) -> Result<Self> {
        let tag = buf.get_u8();
        debug_assert_eq!(tag, 0xfe);

        // SAFE: auth plugins are ASCII only
        #[allow(unsafe_code)]
        let name = unsafe { buf.get_str_nul_unchecked()? };

        if buf.ends_with(&[0]) {
            // if this terminates in a NUL; drop the NUL
            buf.truncate(buf.len() - 1);
        }

        let plugin_data = buf.chain(Bytes::new());

        let plugin = AuthPlugin::parse(&*name)?;

        Ok(Self { plugin, plugin_data })
    }
}
