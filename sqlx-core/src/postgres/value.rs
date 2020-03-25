use crate::error::UnexpectedNullError;
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
    type_oid: u32,
    data: Option<PgData<'c>>,
}

impl<'c> PgValue<'c> {
    /// Gets the binary or text data for this value; or, `UnexpectedNullError` if this
    /// is a `NULL` value.
    pub(crate) fn try_get(&self) -> crate::Result<Postgres, PgData<'c>> {
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

    pub(crate) fn null(type_oid: u32) -> Self {
        Self {
            type_oid,
            data: None,
        }
    }

    pub(crate) fn bytes(type_oid: u32, buf: &'c [u8]) -> Self {
        Self {
            type_oid,
            data: Some(PgData::Binary(buf)),
        }
    }

    pub(crate) fn utf8(type_oid: u32, buf: &'c [u8]) -> crate::Result<Postgres, Self> {
        Ok(Self {
            type_oid,
            data: Some(PgData::Text(from_utf8(&buf).map_err(crate::Error::decode)?)),
        })
    }

    pub(crate) fn str(type_oid: u32, s: &'c str) -> Self {
        Self {
            type_oid,
            data: Some(PgData::Text(s)),
        }
    }
}

impl<'c> RawValue<'c> for PgValue<'c> {
    type Database = Postgres;

    fn type_info(&self) -> PgTypeInfo {
        PgTypeInfo::with_oid(self.type_oid)
    }
}
