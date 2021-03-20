use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::io::PgWriteExt;

#[derive(Debug)]
pub(crate) struct Startup<'a>(pub(crate) &'a [(&'a str, Option<&'a str>)]);

impl Serialize<'_> for Startup<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.write_len_prefixed(|buf| {
            // The protocol version number. The most significant 16 bits are the
            // major version number (3 for the protocol described here). The least
            // significant 16 bits are the minor version number (0
            // for the protocol described here)
            buf.extend(&196_608_i32.to_be_bytes());

            // For each startup parameter, write the name and value
            // as NUL-terminated strings
            for (name, value) in self.0 {
                if let Some(value) = value {
                    write_startup_param(buf, name, value);
                }
            }

            // Followed by a trailing NUL
            buf.push(0);

            Ok(())
        })
    }
}

fn write_startup_param(buf: &mut Vec<u8>, name: &str, value: &str) {
    buf.reserve(name.len() + value.len() + 2);
    buf.extend(name.as_bytes());
    buf.push(0);
    buf.extend(value.as_bytes());
    buf.push(0);
}

#[cfg(test)]
mod tests {
    use super::{Serialize, Startup};

    #[test]
    fn should_encode_startup() {
        let mut buf = Vec::new();
        let m = Startup(&[("user", Some("postgres")), ("database", Some("postgres"))]);

        m.serialize(&mut buf).unwrap();

        assert_eq!(buf, b"\0\0\0)\0\x03\0\0user\0postgres\0database\0postgres\0\0");
    }
}
