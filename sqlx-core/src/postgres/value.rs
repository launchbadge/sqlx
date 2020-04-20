use std::fmt::{self, Display, Formatter};
use std::str::from_utf8;

use bytes::Bytes;

use crate::decode::{Error as DecodeError, UnexpectedNullError};
use crate::ext::ustr::UStr;
use crate::postgres::Postgres;
use crate::value::RawValue;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PgValueFormat {
    Text = 0,
    Binary = 1,
}

// TODO: Do **not** derive(Debug)

/// Implementation of [`RawValue`] for PostgreSQL.
#[derive(Debug)]
pub struct PgRawValue<'r> {
    pub(crate) value: Option<&'r [u8]>,
    pub(crate) format: PgValueFormat,
}

impl<'r> PgRawValue<'r> {
    #[inline]
    pub(crate) fn format(&self) -> PgValueFormat {
        self.format
    }

    #[inline]
    pub(crate) fn is_null(&self) -> bool {
        self.value.is_none()
    }

    pub(crate) fn as_bytes(&self) -> Result<&'r [u8], DecodeError> {
        match &self.value {
            Some(v) => Ok(v),
            None => Err(UnexpectedNullError.into()),
        }
    }

    pub(crate) fn as_str(&self) -> Result<&'r str, DecodeError> {
        Ok(from_utf8(self.as_bytes()?)?)
    }
}

impl<'r> RawValue<'r> for PgRawValue<'r> {
    type Database = Postgres;
}
