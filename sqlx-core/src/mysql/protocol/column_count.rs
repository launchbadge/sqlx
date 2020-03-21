use byteorder::LittleEndian;

use crate::mysql::io::BufExt;
use crate::mysql::MySql;

#[derive(Debug)]
pub struct ColumnCount {
    pub columns: u64,
}

impl ColumnCount {
    pub(crate) fn read(mut buf: &[u8]) -> crate::Result<MySql, Self> {
        let columns = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0);

        Ok(Self { columns })
    }
}
