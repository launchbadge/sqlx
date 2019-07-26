use super::Encode;
use byteorder::{BigEndian, ByteOrder};
use std::io;

#[derive(Debug)]
pub struct StartupMessage<'a> {
    params: &'a [(&'a str, &'a str)],
}

impl<'a> StartupMessage<'a> {
    #[inline]
    pub fn new(params: &'a [(&'a str, &'a str)]) -> Self {
        Self { params }
    }

    #[inline]
    pub fn params(&self) -> &'a [(&'a str, &'a str)] {
        self.params
    }
}

impl<'a> Encode for StartupMessage<'a> {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        let pos = buf.len();
        buf.extend_from_slice(&(0 as u32).to_be_bytes()); // skip over len
        buf.extend_from_slice(&3_u16.to_be_bytes()); // major version
        buf.extend_from_slice(&0_u16.to_be_bytes()); // minor version

        for (name, value) in self.params {
            buf.extend_from_slice(name.as_bytes());
            buf.push(0);
            buf.extend_from_slice(value.as_bytes());
            buf.push(0);
        }

        buf.push(0);

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        BigEndian::write_u32(&mut buf[pos..], len as u32);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Encode, StartupMessage};
    use std::io;

    const STARTUP_MESSAGE: &[u8] = b"\0\0\0)\0\x03\0\0user\0postgres\0database\0postgres\0\0";

    #[test]
    fn it_encodes_startup_message() -> io::Result<()> {
        let message = StartupMessage::new(&[("user", "postgres"), ("database", "postgres")]);

        let mut buf = Vec::new();
        message.encode(&mut buf)?;

        assert_eq!(&*buf, STARTUP_MESSAGE);

        Ok(())
    }
}
