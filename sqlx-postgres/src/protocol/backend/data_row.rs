use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct DataRow {
    pub(crate) values: Vec<Option<Bytes>>,
}

impl Deserialize<'_> for DataRow {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let cnt = buf.get_u16() as usize;

        let mut values = Vec::with_capacity(cnt);

        for _ in 0..cnt {
            // length of the column value, in bytes (this count does not include itself)
            // can be zero. as a special case, -1 indicates a NULL column value
            // no value bytes follow in the NULL case
            let length = buf.get_i32();

            if length < 0 {
                values.push(None);
            } else {
                values.push(Some(buf.split_to(length as usize)));
            }
        }

        Ok(Self { values })
    }
}
