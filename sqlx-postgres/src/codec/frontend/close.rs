use crate::io::{put_length_prefixed, put_portal_name, put_statement_name};
use sqlx_core::{error::Error, io::Encode};

const CLOSE_PORTAL: u8 = b'P';
const CLOSE_STATEMENT: u8 = b'S';

#[derive(Debug)]
pub(crate) enum Close {
    Statement(u32),
    Portal(u32),
}

impl Encode<'_> for Close {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        // 15 bytes for 1-digit statement/portal IDs
        buf.reserve(20);
        buf.push(b'C');

        put_length_prefixed(buf, true, |buf| {
            match self {
                Close::Statement(id) => {
                    buf.push(CLOSE_STATEMENT);
                    put_statement_name(buf, Some(*id));
                }

                Close::Portal(id) => {
                    buf.push(CLOSE_PORTAL);
                    put_portal_name(buf, Some(*id));
                }
            }

            Ok(())
        })
    }
}
