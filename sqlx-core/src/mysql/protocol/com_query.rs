use byteorder::LittleEndian;

use crate::io::BufMut;
use crate::mysql::io::BufMutExt;
use crate::mysql::protocol::{Capabilities, Encode};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_com_query.html
#[derive(Debug)]
pub struct ComQuery<'a> {
    pub query: &'a str,
}

impl Encode for ComQuery<'_> {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_QUERY : int<1>
        buf.put_u8(0x03);

        // query : string<EOF>
        buf.put_str(self.query);
    }
}
