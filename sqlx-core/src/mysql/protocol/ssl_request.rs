use byteorder::LittleEndian;

use crate::io::BufMut;
use crate::mysql::io::BufMutExt;
use crate::mysql::protocol::{AuthPlugin, Capabilities, Encode};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_connection_phase_packets_protocol_handshake_response.html
// https://dev.mysql.com/doc/internals/en/connection-phase-packets.html#packet-Protocol::SSLRequest
#[derive(Debug)]
pub struct SslRequest {
    pub max_packet_size: u32,
    pub client_collation: u8,
}

impl Encode for SslRequest {
    fn encode(&self, buf: &mut Vec<u8>, capabilities: Capabilities) {
        // SSL must be set or else it makes no sense to ask for an upgrade
        assert!(
            capabilities.contains(Capabilities::SSL),
            "SSL bit must be set for Capabilities"
        );

        // client capabilities : int<4>
        buf.put_u32::<LittleEndian>(capabilities.bits() as u32);

        // max packet size : int<4>
        buf.put_u32::<LittleEndian>(self.max_packet_size);

        // client character collation : int<1>
        buf.put_u8(self.client_collation);

        // reserved : string<23>
        buf.advance(23);
    }
}
