use crate::io::{Buf, BufMut};
use byteorder::NetworkEndian;

pub struct SslRequest;

impl SslRequest {
    pub fn encode(buf: &mut Vec<u8>) {
        // packet length: 8 bytes including self
        buf.put_u32::<NetworkEndian>(8);
        // 1234 in high 16 bits, 5679 in low 16
        buf.put_u32::<NetworkEndian>((1234 << 16) | 5679);
    }
}

#[test]
fn test_ssl_request() {
    use crate::io::Buf;

    let mut buf = Vec::new();
    SslRequest::encode(&mut buf);

    assert_eq!((&buf[..]).get_u32::<NetworkEndian>().unwrap(), 80877103);
}
