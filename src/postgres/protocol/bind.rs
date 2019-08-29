use super::Encode;
use crate::io::BufMut;
use byteorder::{BigEndian, ByteOrder, NetworkEndian};

pub struct Bind<'a> {
    /// The name of the destination portal (an empty string selects the unnamed portal).
    pub portal: &'a str,

    /// The name of the source prepared statement (an empty string selects the unnamed prepared statement).
    pub statement: &'a str,

    /// The parameter format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no parameters or that the parameters all use the
    /// default format (text); or one, in which case the specified format code is applied to all
    /// parameters; or it can equal the actual number of parameters.
    pub formats: &'a [i16],

    pub values_len: i16,
    pub values: &'a [u8],

    /// The result-column format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no result columns or that the
    /// result columns should all use the default format (text); or one, in which
    /// case the specified format code is applied to all result columns (if any);
    /// or it can equal the actual number of result columns of the query.
    pub result_formats: &'a [i16],
}

impl Encode for Bind<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'B');

        let pos = buf.len();
        buf.put_i32::<NetworkEndian>(0); // skip over len

        buf.put_str_nul(self.portal);
        buf.put_str_nul(self.statement);

        buf.put_i16::<NetworkEndian>(self.formats.len() as i16);

        for &format in self.formats {
            buf.put_i16::<NetworkEndian>(format);
        }

        buf.put_i16::<NetworkEndian>(self.values_len);

        buf.extend_from_slice(self.values);

        buf.put_i16::<NetworkEndian>(self.result_formats.len() as i16);

        for &format in self.result_formats {
            buf.put_i16::<NetworkEndian>(format);
        }

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        NetworkEndian::write_i32(&mut buf[pos..], len as i32);
    }
}
