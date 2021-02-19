use std::fmt::{self, Debug, Formatter};

use sqlx_core::io::Serialize;
use sqlx_core::{Arguments, Result};

use super::Command;
use crate::{MySql, MySqlOutput, MySqlTypeId, MySqlTypeInfo};

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/mysql__com_8h.html#a3e5e9e744ff6f7b989a604fd669977da
const NO_CURSOR: u8 = 0;

/// Asks the server to execute a prepared statement as identified.
///
/// <https://dev.mysql.com/doc/internals/en/com-stmt-execute.html>
/// <https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_com_stmt_execute.html>
/// <https://mariadb.com/kb/en/com_stmt_execute/>
///
pub(crate) struct Execute<'a, 'x> {
    pub(crate) statement: u32,
    pub(crate) parameters: &'x [MySqlTypeInfo],
    pub(crate) arguments: &'a Arguments<'a, MySql>,
}

impl Debug for Execute<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Execute").field("statement", &self.statement).finish()
    }
}

impl Serialize<'_> for Execute<'_, '_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(0x17);
        buf.extend_from_slice(&self.statement.to_le_bytes());
        buf.push(NO_CURSOR);

        // number of times to execute the statement; can only be 1
        buf.extend_from_slice(&1_u32.to_le_bytes());

        // iterate through each (parameter, value) pair in the statement
        // with the goal to encode each value to the buffer

        let mut out = MySqlOutput::new(buf, self.parameters.len());
        let mut args = self.arguments.positional();

        for param in self.parameters {
            match args.next() {
                Some(argument) => {
                    argument.encode(param, &mut out)?;
                    out.declare(argument.type_id(param));
                }

                None => {
                    // if we run out of values, start sending NULL for
                    // each subsequent parameter
                    out.null();
                    out.declare(MySqlTypeId::NULL);
                }
            }
        }

        Ok(())
    }
}

impl Command for Execute<'_, '_> {}
