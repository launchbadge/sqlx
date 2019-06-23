use crate::{Decode, Encode};
use bytes::Bytes;
use md5::{Digest, Md5};
use std::io;

pub struct PasswordMessage {
    password: Bytes,
}

impl PasswordMessage {
    /// Create a `PasswordMessage` with an unecrypted password.
    pub fn cleartext(password: impl AsRef<str>) -> Self {
        Self { password: Bytes::from(password.as_ref()) }
    }

    /// Create a `PasswordMessage` by hasing the password, user, and salt together using MD5.
    pub fn md5(password: impl AsRef<str>, user: impl AsRef<str>, salt: &[u8; 4]) -> Self {
        let credentials =
            hex::encode(Md5::new().chain(password.as_ref()).chain(user.as_ref()).result());

        let salted = hex::encode(Md5::new().chain(credentials).chain(salt).result());

        let mut password = Vec::with_capacity(3 + salted.len());
        password.copy_from_slice(b"md5");
        password.copy_from_slice(salted.as_bytes());

        Self { password: Bytes::from(password) }
    }

    /// The password (encrypted, if requested).
    pub fn password(&self) -> &[u8] {
        &self.password
    }
}

impl Decode for PasswordMessage {
    fn decode(src: Bytes) -> io::Result<Self>
    where
        Self: Sized,
    {
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
        buf.copy_from_slice(&(self.password.len() + 4).to_be_bytes());
        buf.copy_from_slice(&self.password);

        Ok(())
    }
}

// TODO: Encode and Decode tests
