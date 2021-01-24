use std::fmt::Write;

use md5::{Digest, Md5};
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

                    let mut hasher = Md5::new();

                    hasher.update(password);
                    hasher.update(username);

                    let mut output = String::with_capacity(35);

                    let _ = write!(output, "{:x}", hasher.finalize_reset());

                    hasher.update(&output);
                    hasher.update(salt);

                    output.clear();

                    let _ = write!(output, "md5{:x}", hasher.finalize());

                    buf.write_str_nul(&output);
                }
            }
        });

        Ok(())
    }
}
