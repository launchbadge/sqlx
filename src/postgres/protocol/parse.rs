use super::{BufMut, Encode};

pub struct Parse<'a> {
    pub portal: &'a str,
    pub query: &'a str,
    pub param_types: &'a [i32],
}

impl Encode for Parse<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'P');

        // len + portal + nul + query + null + len(param_types) + param_types
        let len = 4 + self.portal.len() + 1 + self.query.len() + 1 + 2 + self.param_types.len() * 4;
        buf.put_int_32(len as i32);

        buf.put_str(self.portal);
        buf.put_str(self.query);

        buf.put_array_int_32(&self.param_types);
    }
}
