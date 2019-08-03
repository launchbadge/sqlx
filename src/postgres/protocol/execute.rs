use super::{BufMut, Encode};

pub struct Execute<'a> {
    /// The name of the portal to execute (an empty string selects the unnamed portal).
    pub portal: &'a str,

    /// Maximum number of rows to return, if portal contains a query
    /// that returns rows (ignored otherwise). Zero denotes “no limit”.
    pub limit: i32,
}

impl Encode for Execute<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'E');
        // len + nul + len(string) + limit
        buf.put_int_32((4 + 1 + self.portal.len() + 4) as i32);
        buf.put_str(&self.portal);
        buf.put_int_32(self.limit);
    }
}
