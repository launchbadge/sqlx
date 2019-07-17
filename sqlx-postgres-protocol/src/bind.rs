use crate::Encode;
use byteorder::{BigEndian, ByteOrder};
use std::io;

#[derive(Debug)]
pub struct Bind<'a> {
    // The name of the destination portal (an empty string selects the unnamed portal).
    portal: &'a str,

    // The name of the source prepared statement (an empty string selects the unnamed prepared statement).
    statement: &'a str,

    // The parameter format codes.
    formats: &'a [i16],

    // The values of the parameters.
    // Arranged as: [len][size_0][value_0][size_1][value_1] etc...
    buffer: &'a [u8],

    // The result-column format codes.
    result_formats: &'a [i16],
}

impl<'a> Bind<'a> {
    pub fn new(
        portal: &'a str,
        statement: &'a str,
        formats: &'a [i16],
        buffer: &'a [u8],
        result_formats: &'a [i16],
    ) -> Self {
        Self {
            portal,
            statement,
            formats,
            buffer,
            result_formats,
        }
    }
}

impl<'a> Encode for Bind<'a> {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'B');

        let pos = buf.len();
        buf.extend_from_slice(&[0, 0, 0, 0]);

        // portal
        buf.extend_from_slice(self.portal.as_bytes());
        buf.push(b'\0');

        // statement
        buf.extend_from_slice(self.statement.as_bytes());
        buf.push(b'\0');

        // formats.len
        buf.extend_from_slice(&(self.formats.len() as i16).to_be_bytes());

        // formats
        for format in self.formats {
            buf.extend_from_slice(&format.to_be_bytes());
        }

        // values
        buf.extend_from_slice(&self.buffer);

        // result_formats.len
        buf.extend_from_slice(&(self.result_formats.len() as i16).to_be_bytes());

        // result_formats
        for format in self.result_formats {
            buf.extend_from_slice(&format.to_be_bytes());
        }

        let len = buf.len() - pos;
        BigEndian::write_u32(&mut buf[pos..], len as u32);

        Ok(())
    }
}
