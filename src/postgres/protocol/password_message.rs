use super::{BufMut, Encode};
use md5::{Digest, Md5};

#[derive(Debug)]
pub enum PasswordMessage<'a> {
    Cleartext(&'a str),
    Md5 {
        password: &'a str,
        user: &'a str,
        salt: [u8; 4],
    },
}

impl Encode for PasswordMessage<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'p');

        match self {
            PasswordMessage::Cleartext(s) => {
                // len + password + nul
                buf.put_int_32((4 + s.len() + 1) as i32);
                buf.put_str(s);
            }

            PasswordMessage::Md5 {
                password,
                user,
                salt,
            } => {
                let mut hasher = Md5::new();

                hasher.input(password);
                hasher.input(user);

                let credentials = hex::encode(hasher.result_reset());

                hasher.input(credentials);
                hasher.input(salt);

                let salted = hex::encode(hasher.result());

                // len + "md5" + (salted)
                buf.put_int_32((4 + 3 + salted.len()) as i32);

                buf.put(b"md5");
                buf.put(salted.as_bytes());
            }
        }
    }
}
