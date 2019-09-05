use crate::{
    io::BufMut,
    mariadb::{
        io::BufMutExt,
        protocol::{Capabilities, Encode},
    },
};

#[derive(Default, Debug)]
pub struct AuthenticationSwitchRequest<'a> {
    pub auth_plugin_name: &'a str,
    pub auth_plugin_data: &'a [u8],
}

impl Encode for AuthenticationSwitchRequest<'_> {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        buf.put_u8(0xFE);
        buf.put_str_nul(&self.auth_plugin_name);
        buf.put_bytes(&self.auth_plugin_data);
    }
}
