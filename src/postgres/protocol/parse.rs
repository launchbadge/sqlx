use super::Encode;
use crate::io::BufMut;
use byteorder::NetworkEndian;

pub struct Parse<'a> {
    pub portal: &'a str,
    pub query: &'a str,
    pub param_types: &'a [u32],
}

impl Encode for Parse<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'P');

        // len + portal + nul + query + null + len(param_types) + param_types
        let len = 4 + self.portal.len() + 1 + self.query.len() + 1 + 2 + self.param_types.len() * 4;
        buf.put_i32::<NetworkEndian>(len as i32);

        buf.put_str_nul(self.portal);
        buf.put_str_nul(self.query);

        buf.put_i16::<NetworkEndian>(self.param_types.len() as i16);

        for &type_ in self.param_types {
            buf.put_u32::<NetworkEndian>(type_);
        }
    }
}
