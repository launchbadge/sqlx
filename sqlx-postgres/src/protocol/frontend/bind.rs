use std::fmt::{self, Debug, Formatter};

use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::io::PgWriteExt;
use crate::protocol::frontend::{PortalRef, StatementId};
use crate::{PgArguments, PgOutput, PgRawValueFormat, PgTypeInfo};
use sqlx_core::encode::IsNull;

pub(crate) struct Bind<'a> {
    pub(crate) portal: PortalRef,
    pub(crate) statement: StatementId,
    pub(crate) parameters: &'a [PgTypeInfo],
    pub(crate) arguments: &'a PgArguments<'a>,
}

impl Debug for Bind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bind")
            .field("statement", &self.statement)
            .field("portal", &self.portal)
            .finish()
    }
}

impl Serialize<'_> for Bind<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'B');
        buf.write_len_prefixed(|buf| {
            self.portal.serialize(buf)?;
            self.statement.serialize(buf)?;

            // the parameter format codes, each must presently be zero (text) or one (binary)
            // can use one to indicate that all parameters use that format
            write_i16_arr(buf, &[PgRawValueFormat::Binary as i16]);

            // note: this should have been checked in parse
            debug_assert!(!(self.arguments.len() >= (u16::MAX as usize)));

            // note: named arguments should have been converted to positional before this point
            debug_assert_eq!(self.arguments.num_named(), 0);

            buf.extend(&(self.parameters.len() as i16).to_be_bytes());

            let mut out = PgOutput::new(buf);
            let mut args = self.arguments.positional();

            for param in self.parameters {
                // reserve space to write the prefixed length of the value
                let offset = out.buffer().len();
                out.buffer().extend_from_slice(&[0; 4]);

                let prev_len = out.buffer().len();
                let null = args
                    .next()
                    .map(|argument| argument.encode(param, &mut out))
                    .transpose()?
                    // if we run out of values, start sending NULL for
                    .unwrap_or(IsNull::Yes);

                let len = match null {
                    // NULL is encoded as a -1 for the length
                    IsNull::Yes => {
                        // no data *should* have been written to the buffer if we were told the expression is NULL
                        debug_assert_eq!(prev_len, out.buffer().len());

                        -1_i32
                    }

                    IsNull::No => {
                        // prefixed length does not include the length in the length
                        // unlike the regular "prefixed length" bytes protocol type
                        (out.buffer().len() - offset - 4) as i32
                    }
                };

                // write the len to the beginning of the value
                out.buffer()[offset..(offset + 4)].copy_from_slice(&len.to_be_bytes());
            }

            // the result format codes, each must presently be zero (text) or one (binary)
            // can use one to indicate that all results use that format
            write_i16_arr(buf, &[PgRawValueFormat::Binary as i16]);

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
