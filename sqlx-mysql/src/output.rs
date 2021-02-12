use bytes::BufMut;

use crate::MySqlTypeId;

// https://dev.mysql.com/doc/internals/en/com-stmt-execute.html

// 'x: single execution
pub struct MySqlOutput<'x> {
    buffer: &'x mut Vec<u8>,
}

impl<'x> MySqlOutput<'x> {
    pub(crate) fn buffer(&mut self) -> &mut Vec<u8> {
        self.buffer
    }
}
