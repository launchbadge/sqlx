use std::fmt::{self, Debug, Formatter};

use sqlx_core::io::{Serialize, WriteExt};
use sqlx_core::Result;

use crate::io::PgWriteExt;
use crate::protocol::frontend::StatementId;
use crate::{PgArguments, PgTypeId};

pub(crate) struct Parse<'a> {
    pub(crate) statement: StatementId,
    pub(crate) sql: &'a str,
    pub(crate) arguments: &'a PgArguments<'a>,
}

impl Serialize<'_> for Parse<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'P');
        buf.write_len_prefixed(|buf| {
            self.statement.serialize(buf)?;

            buf.write_str_nul(self.sql);

            // TODO: return a proper error
            assert!(!(self.arguments.len() >= (u16::MAX as usize)));

            // note: named arguments should have been converted to positional before this point
            debug_assert_eq!(self.arguments.num_named(), 0);

            buf.extend(&(self.arguments.len() as u16).to_be_bytes());

            for arg in self.arguments.positional() {
                let oid = match arg.type_id() {
                    PgTypeId::Oid(oid) => oid,
                    PgTypeId::Name(_) => todo!(),
                };

                buf.extend(&oid.to_be_bytes());
            }

            Ok(())
        })
    }
}

impl Debug for Parse<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parse").field("statement", &self.statement).field("sql", &self.sql).finish()
    }
}
