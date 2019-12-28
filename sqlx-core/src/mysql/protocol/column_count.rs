use byteorder::LittleEndian;

use crate::io::Buf;
use crate::mysql::io::BufExt;
use crate::mysql::protocol::Decode;

#[derive(Debug)]
pub struct ColumnCount {
    pub columns: u64,
}

impl Decode for ColumnCount {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let columns = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0);

        Ok(Self { columns })
    }
}
