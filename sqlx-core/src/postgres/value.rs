use crate::error::UnexpectedNullError;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgTypeInfo, Postgres};
use crate::value::RawValue;
use std::str::from_utf8;

#[derive(Debug, Copy, Clone)]
pub enum PgData<'c> {
    Binary(&'c [u8]),
    Text(&'c str),
}

#[derive(Debug)]
pub struct PgValue<'c> {
    type_id: TypeId,
    data: Option<PgData<'c>>,
}

impl<'c> PgValue<'c> {
    /// Gets the binary or text data for this value; or, `UnexpectedNullError` if this
    /// is a `NULL` value.
    pub(crate) fn try_get(&self) -> crate::Result<PgData<'c>> {
        match self.data {
            Some(data) => Ok(data),
            None => Err(crate::Error::decode(UnexpectedNullError)),
        }
    }

    /// Gets the binary or text data for this value; or, `None` if this
    /// is a `NULL` value.
    #[inline]
    pub fn get(&self) -> Option<PgData<'c>> {
        self.data
    }

    pub(crate) fn null(type_id: TypeId) -> Self {
        Self {
            type_id,
            data: None,
        }
    }

    pub(crate) fn bytes(type_id: TypeId, buf: &'c [u8]) -> Self {
        Self {
            type_id,
            data: Some(PgData::Binary(buf)),
        }
    }

    pub(crate) fn utf8(type_id: TypeId, buf: &'c [u8]) -> crate::Result<Self> {
        Ok(Self {
            type_id,
            data: Some(PgData::Text(from_utf8(&buf).map_err(crate::Error::decode)?)),
        })
    }

    pub(crate) fn str(type_id: TypeId, s: &'c str) -> Self {
        Self {
            type_id,
            data: Some(PgData::Text(s)),
        }
    }
}

impl<'c> RawValue<'c> for PgValue<'c> {
    type Database = Postgres;

    fn type_info(&self) -> Option<PgTypeInfo> {
        if self.data.is_some() {
            Some(PgTypeInfo::with_oid(self.type_id.0))
        } else {
            None
        }
    }
}
