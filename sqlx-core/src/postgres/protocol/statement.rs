use std::io::Write as _;

use crate::io::BufMut;
use crate::postgres::protocol::Write;

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct StatementId(pub u32);

impl Write for StatementId {
    fn write(&self, buf: &mut Vec<u8>) {
        if self.0 != 0 {
            let _ = write!(buf, "__sqlx_statement_{}\0", self.0);
        } else {
            buf.put_str_nul("");
        }
    }
}
