use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::io::PgBufMutExt;

const CLOSE_PORTAL: u8 = b'P';
const CLOSE_STATEMENT: u8 = b'S';

#[derive(Debug)]
#[allow(dead_code)]
pub enum Close {
    Statement(u32),
    Portal(u32),
}

impl Serialize<'_, ()> for Close {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        // 15 bytes for 1-digit statement/portal IDs
        buf.reserve(20);
        buf.push(b'C');

        buf.write_length_prefixed(|buf| match self {
            Close::Statement(id) => {
                buf.push(CLOSE_STATEMENT);
                buf.write_statement_name(*id);
            }

            Close::Portal(id) => {
                buf.push(CLOSE_PORTAL);
                buf.write_portal_name(Some(*id));
            }
        });

        Ok(())
    }
}
