use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::{PgTypeId, PgTypeInfo};

#[derive(Debug)]
pub(crate) struct ParameterDescription {
    pub(crate) parameters: Vec<PgTypeInfo>,
}

impl Deserialize<'_> for ParameterDescription {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let cnt = buf.get_u16() as usize;
        let mut parameters = Vec::with_capacity(cnt as usize);

        for _ in 0..cnt {
            parameters.push(PgTypeInfo(PgTypeId::Oid(buf.get_u32())));
        }

        Ok(Self { parameters })
    }
}
