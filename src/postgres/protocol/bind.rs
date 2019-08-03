use super::{BufMut, Encode};
use byteorder::{BigEndian, ByteOrder};

pub struct Bind<'a> {
    /// The name of the destination portal (an empty string selects the unnamed portal).
    portal: &'a str,

    /// The name of the source prepared statement (an empty string selects the unnamed prepared statement).
    statement: &'a str,

    /// The parameter format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no parameters or that the parameters all use the
    /// default format (text); or one, in which case the specified format code is applied to all
    /// parameters; or it can equal the actual number of parameters.
    formats: &'a [i16],

    values: &'a [u8],

    /// The result-column format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no result columns or that the
    /// result columns should all use the default format (text); or one, in which
    /// case the specified format code is applied to all result columns (if any);
    /// or it can equal the actual number of result columns of the query.
    result_formats: &'a [i16],
}

impl Encode for Bind<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'B');

        let pos = buf.len();
        buf.put_int_32(0); // skip over len

        buf.put_str(self.portal);
        buf.put_str(self.statement);
        buf.put_array_int_16(&self.formats);
        buf.put(self.values);
        buf.put_array_int_16(&self.result_formats);

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        BigEndian::write_i32(&mut buf[pos..], len as i32);
    }
}
