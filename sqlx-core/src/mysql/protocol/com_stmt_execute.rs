use byteorder::LittleEndian;

use crate::io::BufMut;
use crate::mysql::io::BufMutExt;
use crate::mysql::protocol::{Capabilities, Encode};
use crate::mysql::types::MySqlTypeMetadata;

bitflags::bitflags! {
    // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/mysql__com_8h.html#a3e5e9e744ff6f7b989a604fd669977da
    // https://mariadb.com/kb/en/library/com_stmt_execute/#flag
    pub struct Cursor: u8 {
        const NO_CURSOR = 0;
        const READ_ONLY = 1;
        const FOR_UPDATE = 2;
        const SCROLLABLE = 4;
    }
}

// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_com_stmt_execute.html
#[derive(Debug)]
pub struct ComStmtExecute<'a> {
    pub statement_id: u32,
    pub cursor: Cursor,
    pub params: &'a [u8],
    pub null_bitmap: &'a [u8],
    pub param_types: &'a [MySqlTypeMetadata],
}

impl Encode for ComStmtExecute<'_> {
    fn encode(&self, buf: &mut Vec<u8>, capabilities: Capabilities) {
        // COM_STMT_EXECUTE : int<1>
        buf.put_u8(0x17);

        // statement_id : int<4>
        buf.put_u32::<LittleEndian>(self.statement_id);

        // cursor : int<1>
        buf.put_u8(self.cursor.bits());

        // iterations (always 1) : int<4>
        buf.put_u32::<LittleEndian>(1);

        if !self.param_types.is_empty() {
            // null bitmap : byte<(param_count + 7)/8>
            buf.put_bytes(self.null_bitmap);

            // send type to server (0 / 1) : byte<1>
            buf.put_u8(1);

            for ty in self.param_types {
                // field type : byte<1>
                buf.put_u8(ty.r#type.0);

                // parameter flag : byte<1>
                buf.put_u8(if ty.is_unsigned { 0x80 } else { 0 });
            }

            // byte<n> binary parameter value
            buf.put_bytes(self.params);
        }
    }
}
