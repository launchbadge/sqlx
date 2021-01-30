use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::io::MySqlBufExt;
use crate::protocol::ColumnDefinition;

#[derive(Debug)]
pub(crate) struct Row {
    pub(crate) values: Vec<Option<Bytes>>,
}

impl<'de> Deserialize<'de, &'de [ColumnDefinition]> for Row {
    fn deserialize_with(mut buf: Bytes, columns: &'de [ColumnDefinition]) -> Result<Self> {
        if columns.is_empty() {
            return Ok(Self { values: vec![] });
        }

        let mut values = Vec::with_capacity(columns.len());

        for _ in columns {
            values.push(if buf.get(0).copied() == Some(0xfb) {
                buf.advance(1);
                None
            } else {
                Some(buf.get_bytes_lenenc())
            });
        }

        Ok(Self { values })
    }
}
