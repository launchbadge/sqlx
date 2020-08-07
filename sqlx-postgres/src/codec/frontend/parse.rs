use crate::io::{put_length_prefixed, put_statement_name, put_str};
use sqlx_core::{error::Error, io::Encode};

#[derive(Debug)]
pub(crate) struct Parse<'a> {
    /// The ID of the destination prepared statement.
    /// Can be `None`. This will use a temporary server-side storage that lasts until the next
    /// `Parse` is received.
    pub(crate) statement: Option<u32>,

    /// The query string to be parsed.
    pub(crate) query: &'a str,

    /// The parameter data types specified (could be zero). Note that this is not an
    /// indication of the number of parameters that might appear in the query string,
    /// only the number that the frontend wants to pre-specify types for.
    pub(crate) parameter_types: &'a [u32],
}

impl Encode<'_> for Parse<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        buf.push(b'P');

        put_length_prefixed(buf, true, |buf| {
            put_statement_name(buf, self.statement);
            put_str(buf, self.query);

            if self.parameter_types.len() >= (i16::MAX as usize) {
                return Err(Error::Query("too many parameter types to transmit".into()));
            }

            buf.extend(&(self.parameter_types.len() as i16).to_be_bytes());

            for &oid in self.parameter_types {
                buf.extend(&oid.to_be_bytes());
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_named() {
        const EXPECTED: &[u8] = b"P\0\0\0\x1dsqlx_s_1\0SELECT $1\0\0\x01\0\0\0\x19";

        let mut buf = Vec::new();

        let m = Parse {
            statement: Some(1),
            query: "SELECT $1",
            parameter_types: &[25],
        };

        m.encode(&mut buf);

        assert_eq!(buf, EXPECTED);
    }
}
