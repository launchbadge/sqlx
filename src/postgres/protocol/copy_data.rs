use super::Encode;
use crate::io::BufMut;
use byteorder::NetworkEndian;

// TODO: Implement Decode and think on an optimal representation

/*
# Optimal for Encode
pub struct CopyData<'a> { data: &'a [u8] }

# Optimal for Decode
pub struct CopyData { data: Bytes }

# 1) Two structs (names?)
# 2) "Either" inner abstraction; removes ease of construction for Encode
*/

pub struct CopyData<'a> {
    pub data: &'a [u8],
}

impl Encode for CopyData<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'd');
        // len + nul + len(string)
        buf.put_i32::<NetworkEndian>((4 + 1 + self.data.len()) as i32);
        buf.extend_from_slice(&self.data);
    }
}
