use crate::io::PgWriteExt;
use md5::{Digest, Md5};
use sqlx_core::io::Serialize;
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct Password<'a>(pub(crate) &'a str);

#[derive(Debug)]
pub(crate) struct PasswordMd5<'a> {
    pub(crate) password: &'a str,
    pub(crate) username: &'a str,
    pub(crate) salt: [u8; 4],
}

impl Serialize<'_> for Password<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.reserve(1 + 4 + self.0.len() + 1);
        buf.push(b'p');

        buf.write_len_prefixed(|buf| {
            buf.extend(self.0.as_bytes());
            buf.push(b'\0');

            Ok(())
        })
    }
}

impl Serialize<'_> for PasswordMd5<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.reserve(1 + 4 + 3 + 32 + 1);
        buf.push(b'p');

        buf.write_len_prefixed(|buf| {
            // the actual `PasswordMessage` can be computed in SQL as:
            //      concat('md5', md5(concat(md5(concat(password, username)), random-salt)))

            // keep in mind the md5() function returns its result as a hex string

            let mut hasher = Md5::new();

            hasher.update(self.password);
            hasher.update(self.username);

            let offset = buf.len();
            buf.resize(offset + 32 + 3, 0);

            let hash = hasher.finalize_reset();
            let _ = hex::encode_to_slice(hash.as_slice(), &mut buf[offset..offset + 32]);

            hasher.update(&buf[offset..offset + 32]);
            hasher.update(self.salt);

            buf[offset..offset + 3].copy_from_slice(&b"md5"[..]);

            let hash = hasher.finalize();
            let _ = hex::encode_to_slice(hash.as_slice(), &mut buf[offset + 3..offset + 32 + 3]);

            buf.push(b'\0');

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Password, PasswordMd5, Serialize};

    #[test]
    fn should_serialize() {
        let mut buf = Vec::new();
        let m = Password("password");

        m.serialize(&mut buf).unwrap();

        assert_eq!(buf, b"p\0\0\0\rpassword\0");
    }

    #[test]
    fn should_serialize_md5() {
        let mut buf = Vec::new();
        let m = PasswordMd5 { password: "password", username: "root", salt: [147, 24, 57, 152] };

        m.serialize(&mut buf).unwrap();

        assert_eq!(buf, b"p\0\0\0(md53e2c9d99d49b201ef867a36f3f9ed62c\0");
    }
}
