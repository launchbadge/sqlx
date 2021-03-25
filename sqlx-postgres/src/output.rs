// 'x: execution
#[allow(clippy::module_name_repetitions)]
pub struct PgOutput<'x> {
    buffer: &'x mut Vec<u8>,
}

impl<'x> PgOutput<'x> {
    pub(crate) fn new(buffer: &'x mut Vec<u8>) -> Self {
        Self { buffer }
    }

    pub(crate) fn buffer(&mut self) -> &mut Vec<u8> {
        self.buffer
    }
}
