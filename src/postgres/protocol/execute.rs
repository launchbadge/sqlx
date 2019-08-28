use super::{Encode};
use crate::io::BufMut;
use byteorder::NetworkEndian;

pub struct Execute<'a> {
    /// The name of the portal to execute (an empty string selects the unnamed portal).
    pub portal: &'a str,

    /// Maximum number of rows to return, if portal contains a query
    /// that returns rows (ignored otherwise). Zero denotes “no limit”.
    pub limit: i32,
}

impl Encode for Execute<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'E');
        // len + nul + len(string) + limit
        buf.put_i32::<NetworkEndian>((4 + 1 + self.portal.len() + 4) as i32);
        buf.put_str_nul(&self.portal);
        buf.put_i32::<NetworkEndian>(self.limit);
    }
}
