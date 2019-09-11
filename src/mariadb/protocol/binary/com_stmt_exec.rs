use crate::{
    io::BufMut,
    mariadb::{
        io::BufMutExt,
        protocol::{binary::BinaryProtocol, Capabilities, Encode},
        types::MariaDbTypeMetadata,
    },
};
use byteorder::LittleEndian;

bitflags::bitflags! {
    // https://mariadb.com/kb/en/library/com_stmt_execute/#flag
    pub struct StmtExecFlag: u8 {
        const NO_CURSOR = 0;
        const READ_ONLY = 1;
        const CURSOR_FOR_UPDATE = 2;
        const SCROLLABLE_CURSOR = 4;
    }
}

// https://mariadb.com/kb/en/library/com_stmt_execute
/// Executes a previously prepared statement.
#[derive(Debug)]
pub struct ComStmtExecute<'a> {
    pub statement_id: u32,
    pub flags: StmtExecFlag,
    pub params: &'a [u8],
    pub null: &'a [u8],
    pub param_types: &'a [MariaDbTypeMetadata],
}

impl Encode for ComStmtExecute<'_> {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_STMT_EXECUTE : int<1>
        buf.put_u8(BinaryProtocol::ComStmtExec as u8);

        // statement id : int<4>
        buf.put_u32::<LittleEndian>(self.statement_id);

        // flags : int<1>
        buf.put_u8(self.flags.bits());

        // Iteration count (always 1) : int<4>
        buf.put_u32::<LittleEndian>(1);

        // if (param_count > 0)
        if self.param_types.len() > 0 {
            // null bitmap : byte<(param_count + 7)/8>
            buf.put_bytes(self.null);

            // send type to server (0 / 1) : byte<1>
            buf.put_u8(1);

            // for each parameter :
            for param_type in self.param_types {
                // field type : byte<1>
                buf.put_u8(param_type.field_type.0);

                // parameter flag : byte<1>
                buf.put_u8(param_type.param_flag.bits());
            }

            // for each parameter (i.e param_count times)
            // byte<n> binary parameter value
            buf.put_bytes(self.params);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_exec() {
        let mut buf = Vec::new();

        ComStmtExecute {
            statement_id: 1,
            flags: StmtExecFlag::NO_CURSOR,
            null: &vec![],
            params: &vec![],
            param_types: &vec![],
        }
        .encode(&mut buf, Capabilities::empty());

        // TODO: Add a regression test
    }
}
