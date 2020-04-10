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
    type_info: Option<PgTypeInfo>,
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

    pub(crate) fn null() -> Self {
        Self {
            type_info: None,
            data: None,
        }
    }

    pub(crate) fn bytes(type_info: PgTypeInfo, buf: &'c [u8]) -> Self {
        Self {
            type_info: Some(type_info),
            data: Some(PgData::Binary(buf)),
        }
    }

    pub(crate) fn utf8(type_info: PgTypeInfo, buf: &'c [u8]) -> crate::Result<Self> {
        Ok(Self {
            type_info: Some(type_info),
            data: Some(PgData::Text(from_utf8(&buf).map_err(crate::Error::decode)?)),
        })
    }

    #[cfg(test)]
    pub(crate) fn from_bytes(buf: &'c [u8]) -> Self {
        Self {
            type_info: None,
            data: Some(PgData::Binary(buf)),
        }
    }

    pub(crate) fn from_str(s: &'c str) -> Self {
        Self {
            type_info: None,
            data: Some(PgData::Text(s)),
        }
    }
}

impl<'c> RawValue<'c> for PgValue<'c> {
    type Database = Postgres;

    // The public type_info is used for type compatibility checks
    fn type_info(&self) -> Option<PgTypeInfo> {
        // For TEXT encoding the type defined on the value is unreliable
        if matches!(self.data, Some(PgData::Binary(_))) {
            self.type_info.clone()
        } else {
            None
        }
    }
}
