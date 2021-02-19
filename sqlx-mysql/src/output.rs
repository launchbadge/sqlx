// https://dev.mysql.com/doc/internals/en/com-stmt-execute.html

use crate::MySqlTypeId;

// 'x: single execution
#[allow(clippy::module_name_repetitions)]
pub struct MySqlOutput<'x> {
    buffer: &'x mut Vec<u8>,
    null_offset: usize,
    ty_offset: usize,
    index: usize,
}

impl<'x> MySqlOutput<'x> {
    pub(crate) fn new(buffer: &'x mut Vec<u8>, params: usize) -> Self {
        // reserve space for the NULL bitmap
        let null_offset = buffer.len();
        let null_len = (params + 7) / 8;
        buffer.resize(null_offset + null_len, 0);

        // let MySQL know we are sending parameter types
        buffer.push(1);

        // reserve space for the parameter types
        let ty_offset = buffer.len();
        let ty_len = params * 2;
        buffer.resize(ty_offset + ty_len, 0);

        Self { buffer, null_offset, ty_offset, index: 0 }
    }

    // for use in protocol::Execute
    // do NOT use in the types/ module
    pub(crate) fn declare(&mut self, ty: MySqlTypeId) {
        self.buffer[self.ty_offset] = ty.ty();
        self.buffer[self.ty_offset + 1] = ty.flags();

        self.ty_offset += 2;
        self.index += 1;
    }

    pub(crate) fn null(&mut self) {
        self.buffer[self.null_offset + (self.index / 8)] |= (1 << (self.index % 8)) as u8;
    }

    pub(crate) fn buffer(&mut self) -> &mut Vec<u8> {
        self.buffer
    }
}
