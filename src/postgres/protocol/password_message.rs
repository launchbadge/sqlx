use super::Encode;
use bytes::Bytes;
use md5::{Digest, Md5};
use std::io;

#[derive(Debug)]
pub struct PasswordMessage {
    password: Bytes,
}

impl PasswordMessage {
    /// Create a `PasswordMessage` with an unecrypted password.
    pub fn cleartext(password: &str) -> Self {
        Self {
            password: Bytes::from(password),
        }
    }

    /// Create a `PasswordMessage` by hasing the password, user, and salt together using MD5.
    pub fn md5(password: &str, user: &str, salt: [u8; 4]) -> Self {
        let mut hasher = Md5::new();

        hasher.input(password);
        hasher.input(user);

        let credentials = hex::encode(hasher.result_reset());

        hasher.input(credentials);
        hasher.input(salt);

        let salted = hex::encode(hasher.result());

        let mut password = Vec::with_capacity(3 + salted.len());
        password.extend_from_slice(b"md5");
        password.extend_from_slice(salted.as_bytes());

        Self {
            password: Bytes::from(password),
        }
    }

    /// The password (encrypted, if requested).
    pub fn password(&self) -> &[u8] {
        &self.password
    }
}

impl Encode for PasswordMessage {
    fn size_hint(&self) -> usize {
        self.password.len() + 5
    }

    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'p');
        buf.extend_from_slice(&(self.password.len() + 4).to_be_bytes());
        buf.extend_from_slice(&self.password);

        Ok(())
    }
}
