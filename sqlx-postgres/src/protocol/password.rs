use std::fmt::Write;

use sqlx_core::io::Serialize;
use sqlx_core::io::WriteExt;
use sqlx_core::Result;

use crate::io::PgBufMutExt;

#[derive(Debug)]
pub enum Password<'a> {
    Cleartext(&'a str),

    Md5 { password: &'a str, username: &'a str, salt: [u8; 4] },
}

impl Password<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Password::Cleartext(s) => s.len() + 5,
            Password::Md5 { .. } => 35 + 5,
        }
    }
}

impl Serialize<'_, ()> for Password<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.reserve(1 + 4 + self.len());
        buf.push(b'p');

        buf.write_length_prefixed(|buf| {
            match self {
                Password::Cleartext(password) => {
                    buf.write_str_nul(password);
                }

                Password::Md5 { username, password, salt } => {
                    // The actual `PasswordMessage` can be comwriteed in SQL as
                    // `concat('md5', md5(concat(md5(concat(password, username)), random-salt)))`.

                    // Keep in mind the md5() function returns its result as a hex string.

                    let digest = md5::compute(password);
                    let digest = md5::compute(username);

                    let mut outwrite = String::with_capacity(35);

                    let _ = write!(outwrite, "{:x}", digest);

                    let digest = md5::compute(&outwrite);
                    let digest = md5::compute(salt);

                    outwrite.clear();

                    let _ = write!(outwrite, "md5{:x}", digest);

                    buf.write_str_nul(&outwrite);
                }
            }
        });

        Ok(())
    }
}
