use super::{BufMut, Encode};
use byteorder::{BigEndian, ByteOrder};

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
        buf.put_byte(b'B');

        let pos = buf.len();
        buf.put_int_32(0); // skip over len

        buf.put_str(self.portal);
        buf.put_str(self.statement);

        buf.put_array_int_16(&self.formats);

        buf.put_int_16(self.values_len);

        buf.put(self.values);

        buf.put_array_int_16(&self.result_formats);

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        BigEndian::write_i32(&mut buf[pos..], len as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::{Bind, BindCollector, BufMut, Encode};

    const BIND: &[u8] = b"B\0\0\0\x18\0\0\0\x01\0\x01\0\x02\0\0\0\x011\0\0\0\x012\0\0";

    #[test]
    fn it_encodes_bind_for_two() {
        let mut buf = Vec::new();

        let mut builder = BindCollector::new();
        builder.add("1");
        builder.add("2");

        let bind = Bind {
            portal: "",
            statement: "",
            formats: builder.formats(),
            values_len: builder.values_len(),
            values: builder.values(),
            result_formats: &[],
        };

        bind.encode(&mut buf);

        assert_eq!(buf, BIND);
    }
}
