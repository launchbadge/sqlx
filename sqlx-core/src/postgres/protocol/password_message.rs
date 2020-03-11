use crate::io::BufMut;
use crate::postgres::protocol::Write;
use byteorder::NetworkEndian;
use md5::{Digest, Md5};

pub(crate) enum PasswordMessage<'a> {
    ClearText(&'a str),

    Md5 {
        password: &'a str,
        user: &'a str,
        salt: [u8; 4],
    },
}

impl Write for PasswordMessage<'_> {
    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(b'p');

        match self {
            PasswordMessage::ClearText(s) => {
                // len + password + nul
                buf.put_u32::<NetworkEndian>((4 + s.len() + 1) as u32);
                buf.put_str_nul(s);
            }

            PasswordMessage::Md5 {
                password,
                user,
                salt,
            } => {
                let mut hasher = Md5::new();

                hasher.input(password);
                hasher.input(user);

                let credentials = format!("{:x}", hasher.result_reset());

                hasher.input(credentials);
                hasher.input(salt);

                let salted = format!("{:x}", hasher.result());

                // len + "md5" + (salted)
                buf.put_u32::<NetworkEndian>((4 + 3 + salted.len() + 1) as u32);

                buf.extend_from_slice(b"md5");
                buf.extend_from_slice(salted.as_bytes());
                buf.push(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PasswordMessage, Write};

    const PASSWORD_CLEAR: &[u8] = b"p\0\0\0\rpassword\0";
    const PASSWORD_MD5: &[u8] = b"p\0\0\0(md53e2c9d99d49b201ef867a36f3f9ed62c\0";

    #[test]
    fn it_writes_password_clear() {
        let mut buf = Vec::new();
        let m = PasswordMessage::ClearText("password");

        m.write(&mut buf);

        assert_eq!(buf, PASSWORD_CLEAR);
    }

    #[test]
    fn it_writes_password_md5() {
        let mut buf = Vec::new();
        let m = PasswordMessage::Md5 {
            password: "password",
            user: "root",
            salt: [147, 24, 57, 152],
        };

        m.write(&mut buf);

        assert_eq!(buf, PASSWORD_MD5);
    }
}
