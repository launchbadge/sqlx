use super::{BufMut, Encode};
use byteorder::{BigEndian, ByteOrder};

pub struct StartupMessage<'a> {
    pub params: &'a [(&'a str, &'a str)],
}

impl Encode for StartupMessage<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        let pos = buf.len();
        buf.put_int_32(0); // skip over len

        // protocol version number (3.0)
        buf.put_int_32(196608);

        for (name, value) in self.params {
            buf.put_str(name);
            buf.put_str(value);
        }

        buf.put_byte(0);

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        BigEndian::write_i32(&mut buf[pos..], len as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::{BufMut, Encode, StartupMessage};

    const STARTUP_MESSAGE: &[u8] = b"\0\0\0)\0\x03\0\0user\0postgres\0database\0postgres\0\0";

    #[test]
    fn it_encodes_startup_message() {
        let mut buf = Vec::new();
        let m = StartupMessage {
            params: &[("user", "postgres"), ("database", "postgres")],
        };

        m.encode(&mut buf);

        assert_eq!(buf, STARTUP_MESSAGE);
    }
}
