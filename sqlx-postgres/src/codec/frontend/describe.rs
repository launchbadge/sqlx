use crate::io::{put_length_prefixed, put_portal_name, put_statement_name};
use sqlx_core::{error::Error, io::Encode};

const DESCRIBE_PORTAL: u8 = b'P';
const DESCRIBE_STATEMENT: u8 = b'S';

// [Describe] will emit both a [RowDescription] and a [ParameterDescription] message

#[derive(Debug)]
pub(crate) enum Describe {
    Statement(Option<u32>),
    Portal(Option<u32>),
}

impl Encode<'_> for Describe {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        // 15 bytes for 1-digit statement/portal IDs
        buf.reserve(20);
        buf.push(b'D');

        put_length_prefixed(buf, true, |buf| {
            match self {
                Describe::Statement(id) => {
                    buf.push(DESCRIBE_STATEMENT);
                    put_statement_name(buf, *id);
                }

                Describe::Portal(id) => {
                    buf.push(DESCRIBE_PORTAL);
                    put_portal_name(buf, *id);
                }
            }

            Ok(())
        })
    }
}
