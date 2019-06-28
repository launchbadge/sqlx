use crate::Encode;
use bytes::{BufMut, Bytes, BytesMut};
use std::io;

#[derive(Debug)]
pub struct StartupMessage {
    // (major, minor)
    version: (u16, u16),
    params: Bytes,
}

impl StartupMessage {
    pub fn builder() -> StartupMessageBuilder {
        StartupMessageBuilder::new()
    }

    pub fn version(&self) -> (u16, u16) {
        self.version
    }

    pub fn params(&self) -> StartupMessageParams<'_> {
        StartupMessageParams(&*self.params)
    }
}

impl Encode for StartupMessage {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        let len = self.params.len() + 8;
        buf.reserve(len);
        buf.put_u32_be(len as u32);
        buf.put_u16_be(self.version.0);
        buf.put_u16_be(self.version.1);
        buf.put(&self.params);

        Ok(())
    }
}

// TODO: Impl Iterator to iter over params
pub struct StartupMessageParams<'a>(&'a [u8]);

pub struct StartupMessageBuilder {
    // (major, minor)
    version: (u16, u16),
    params: BytesMut,
}

impl Default for StartupMessageBuilder {
    fn default() -> Self {
        StartupMessageBuilder { version: (3, 0), params: BytesMut::with_capacity(156) }
    }
}

impl StartupMessageBuilder {
    pub fn new() -> Self {
        StartupMessageBuilder::default()
    }

    /// Set the protocol version number. Defaults to `3.0`.
    pub fn version(mut self, major: u16, minor: u16) -> Self {
        self.version = (major, minor);
        self
    }

    pub fn param(mut self, name: &str, value: &str) -> Self {
        self.params.reserve(name.len() + value.len() + 2);
        self.params.put(name.as_bytes());
        self.params.put_u8(0);
        self.params.put(value.as_bytes());
        self.params.put_u8(0);

        self
    }

    pub fn build(mut self) -> StartupMessage {
        self.params.reserve(1);
        self.params.put_u8(0);

        StartupMessage { version: self.version, params: self.params.freeze() }
    }
}

#[cfg(test)]
mod tests {
    use super::StartupMessage;
    use crate::Encode;
    use std::io;

    const STARTUP_MESSAGE: &[u8] = b"\0\0\0)\0\x03\0\0user\0postgres\0database\0postgres\0\0";

    #[test]
    fn it_encodes_startup_message() -> io::Result<()> {
        let message = StartupMessage::builder()
            .param("user", "postgres")
            .param("database", "postgres")
            .build();

        let mut buf = Vec::new();
        message.encode(&mut buf)?;

        assert_eq!(&*buf, STARTUP_MESSAGE);

        Ok(())
    }
}
