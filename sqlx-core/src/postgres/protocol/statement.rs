use crate::io::BufMut;
use crate::postgres::protocol::Encode;

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct StatementId(pub u32);

impl Encode for StatementId {
    fn encode(&self, buf: &mut Vec<u8>) {
        if self.0 != 0 {
            buf.put_str("__sqlx_statement_");

            // TODO: Use [itoa]
            buf.put_str_nul(&self.0.to_string());
        } else {
            buf.put_str_nul("");
        }
    }
}
