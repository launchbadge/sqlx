use std::io::Write;

use crate::io::BufMut;
use crate::postgres::protocol::Encode;

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct StatementId(pub u32);

impl Encode for StatementId {
    fn encode(&self, buf: &mut Vec<u8>) {
        if self.0 != 0 {
            let _ = write!(buf, "__sqlx_statement_{}\0", self.0);
        } else {
            buf.put_str_nul("");
        }
    }
}
