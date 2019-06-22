use bytes::Bytes;
use std::io;
use crate::{Encode,  Decode};

pub struct PasswordMessage {
    password: Bytes,
}

impl PasswordMessage {
    pub fn cleartext(s: impl AsRef<str>) -> Self {
        // TODO
        unimplemented!()
    }

    pub fn md5(s: impl AsRef<str>) -> Self {
        // TODO
        unimplemented!()
    }

    /// The password (encrypted, if requested).
    pub fn password(&self) -> &[u8] {
        &self.password
    }
}

impl Decode for PasswordMessage {
    fn decode(src: Bytes) -> io::Result<Self> where
        Self: Sized {
        // There is only one field, the password, and it's not like we can
        // decrypt it if it was encrypted
        Ok(PasswordMessage { password: src })
    }
}

impl Encode for PasswordMessage {
    fn size_hint(&self) -> usize {
        self.password.len() + 5
    }

    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'p');
        buf.copy_from_slice((self.password.len() + 4).to_be_bytes());
        buf.copy_from_slice(&self.password);

        Ok(())
    }
}

// TODO: Encode and Decode tests
