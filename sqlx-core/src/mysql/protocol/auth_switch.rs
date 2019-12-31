use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::protocol::{AuthPlugin, Capabilities, Decode, Status};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_connection_phase_packets_protocol_auth_switch_request.html
#[derive(Debug)]
pub struct AuthSwitch {
    pub auth_plugin: AuthPlugin,
    pub auth_plugin_data: Box<[u8]>,
}

impl Decode for AuthSwitch {
    fn decode(mut buf: &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let header = buf.get_u8()?;
        if header != 0xFE {
            return Err(protocol_err!(
                "expected AUTH SWITCH (0xFE); received 0x{:X}",
                header
            ))?;
        }

        let auth_plugin = AuthPlugin::from_opt_str(Some(buf.get_str_nul()?))?;
        let auth_plugin_data = buf.get_bytes(buf.len())?.to_owned().into_boxed_slice();

        Ok(Self {
            auth_plugin_data,
            auth_plugin,
        })
    }
}
