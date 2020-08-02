use crate::io::{put_length_prefixed, put_str};
use sqlx_core::{error::Error, io::Encode};

// To begin a session, a frontend opens a connection to the server and sends a startup message.
// This message includes the names of the user and of the database the user wants to connect to;
// it also identifies the particular protocol version to be used.

// Optionally, the startup message can include additional settings for run-time parameters.

pub(crate) struct Startup<'a>(pub(crate) &'a [(&'a str, Option<&'a str>)]);

impl Encode<'_> for Startup<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        put_length_prefixed(buf, |buf| {
            // The protocol version number.
            //
            // The most significant 16 bits are the major version
            // number (3 for the protocol described here).
            //
            // The least significant 16 bits are the minor version
            // number (0 for the protocol described here).
            buf.extend_from_slice(&0x0003_0000_i32.to_be_bytes());

            for (name, value) in self.0 {
                if let Some(value) = value {
                    put_startup_parameter(buf, name, value);
                }
            }

            // A zero byte is required as a terminator
            // after the last name/value pair.
            buf.push(0);

            Ok(())
        })
    }
}

fn put_startup_parameter(buf: &mut Vec<u8>, name: &str, value: &str) {
    put_str(buf, name);
    put_str(buf, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        const EXPECTED: &[u8] = b"\0\0\0)\0\x03\0\0user\0postgres\0database\0postgres\0\0";

        let mut buf = Vec::new();

        let m = Startup(&[("user", Some("postgres")), ("database", Some("postgres"))]);

        m.encode(&mut buf);

        assert_eq!(buf, EXPECTED);
    }
}

#[cfg(all(test, not(debug_assertions)))]
mod bench {
    use super::*;

    #[bench]
    fn encode(b: &mut test::Bencher) {
        use test::black_box;

        let mut buf = Vec::with_capacity(1024);

        b.iter(|| {
            buf.clear();

            let m = (Startup(&[
                ("user", "postgres"),
                ("database", "postgres"),
                ("DateStyle", "ISO, MDY"),
                ("client_encoding", "UTF8"),
                ("TimeZone", "UTC"),
                ("extra_float_digits", "3"),
            ]));

            m.encode(&mut buf);
        });
    }
}
