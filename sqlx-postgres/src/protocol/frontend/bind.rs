use sqlx_core::io::{Serialize, WriteExt};
use sqlx_core::Result;

use crate::io::PgWriteExt;
use crate::protocol::frontend::{PortalRef, StatementRef};
use crate::PgArguments;

pub(crate) struct Bind<'a> {
    pub(crate) portal: PortalRef,
    pub(crate) statement: StatementRef,
    pub(crate) arguments: &'a PgArguments<'a>,
}

impl Serialize<'_> for Bind<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'B');
        buf.write_len_prefixed(|buf| {
            self.portal.serialize(buf)?;
            self.statement.serialize(buf)?;

            // the parameter format codes, each must presently be zero (text) or one (binary)
            // can use one to indicate that all parameters use that format
            write_i16_arr(buf, &[1]);

            todo!("arguments");

            // the result format codes, each must presently be zero (text) or one (binary)
            // can use one to indicate that all results use that format
            write_i16_arr(buf, &[1]);

            Ok(())
        })
    }
}

fn write_i16_arr(buf: &mut Vec<u8>, arr: &[i16]) {
    buf.extend(&(arr.len() as i16).to_be_bytes());

    for val in arr {
        buf.extend(&val.to_be_bytes());
    }
}
