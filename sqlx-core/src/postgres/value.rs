use std::borrow::Cow;
use std::str::from_utf8;

use bytes::Bytes;

use crate::error::{BoxDynError, UnexpectedNullError};
use crate::postgres::{PgTypeInfo, Postgres};
use crate::value::{Value, ValueRef};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum PgValueFormat {
    Text = 0,
    Binary = 1,
}

/// Implementation of [`ValueRef`] for PostgreSQL.
#[derive(Clone)]
pub struct PgValueRef<'r> {
    pub(crate) value: Option<&'r [u8]>,
    pub(crate) row: Option<&'r Bytes>,
    pub(crate) type_info: PgTypeInfo,
    pub(crate) format: PgValueFormat,
}

/// Implementation of [`Value`] for PostgreSQL.
#[derive(Clone)]
pub struct PgValue {
    pub(crate) value: Option<Bytes>,
    pub(crate) type_info: PgTypeInfo,
    pub(crate) format: PgValueFormat,
}

impl<'r> PgValueRef<'r> {
    pub(crate) fn format(&self) -> PgValueFormat {
        self.format
    }

    pub(crate) fn as_bytes(&self) -> Result<&'r [u8], BoxDynError> {
        match &self.value {
            Some(v) => Ok(v),
            None => Err(UnexpectedNullError.into()),
        }
    }

    pub(crate) fn as_str(&self) -> Result<&'r str, BoxDynError> {
        Ok(from_utf8(self.as_bytes()?)?)
    }
}

impl Value for PgValue {
    type Database = Postgres;

    #[inline]
    fn as_ref(&self) -> PgValueRef<'_> {
        PgValueRef {
            value: self.value.as_deref(),
            row: None,
            type_info: self.type_info.clone(),
            format: self.format,
        }
    }

    fn type_info(&self) -> Option<Cow<'_, PgTypeInfo>> {
        Some(Cow::Borrowed(&self.type_info))
    }

    fn is_null(&self) -> bool {
        self.value.is_none()
    }
}

impl<'r> ValueRef<'r> for PgValueRef<'r> {
    type Database = Postgres;

    fn to_owned(&self) -> PgValue {
        let value = match (self.row, self.value) {
            (Some(row), Some(value)) => Some(row.slice_ref(value)),

            (None, Some(value)) => Some(Bytes::copy_from_slice(value)),

            _ => None,
        };

        PgValue {
            value,
            format: self.format,
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Option<Cow<'_, PgTypeInfo>> {
        Some(Cow::Borrowed(&self.type_info))
    }

    fn is_null(&self) -> bool {
        self.value.is_none()
    }
}
