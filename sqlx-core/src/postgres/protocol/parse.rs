use crate::io::BufMut;
use crate::postgres::protocol::{StatementId, Write};
use byteorder::{ByteOrder, NetworkEndian};

pub struct Parse<'a> {
    pub statement: StatementId,
    pub query: &'a str,
    pub param_types: &'a [u32],
}

impl Write for Parse<'_> {
    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(b'P');

        let pos = buf.len();
        buf.put_i32::<NetworkEndian>(0); // skip over len

        self.statement.write(buf);

        buf.put_str_nul(self.query);

        buf.put_i16::<NetworkEndian>(self.param_types.len() as i16);

        for &type_ in self.param_types {
            buf.put_u32::<NetworkEndian>(type_);
        }

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        NetworkEndian::write_i32(&mut buf[pos..], len as i32);
    }
}
