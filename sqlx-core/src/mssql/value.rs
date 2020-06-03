use std::borrow::Cow;
use std::marker::PhantomData;

use bytes::Bytes;

use crate::database::HasValueRef;
use crate::error::{BoxDynError, UnexpectedNullError};
use crate::mssql::{MsSql, MsSqlTypeInfo};
use crate::value::{Value, ValueRef};

/// Implementation of [`ValueRef`] for MSSQL.
#[derive(Clone)]
pub struct MsSqlValueRef<'r> {
    pub(crate) type_info: MsSqlTypeInfo,
    pub(crate) data: Option<&'r Bytes>,
}

impl<'r> MsSqlValueRef<'r> {
    pub(crate) fn as_bytes(&self) -> Result<&'r [u8], BoxDynError> {
        match &self.data {
            Some(v) => Ok(v),
            None => Err(UnexpectedNullError.into()),
        }
    }
}

impl ValueRef<'_> for MsSqlValueRef<'_> {
    type Database = MsSql;

    fn to_owned(&self) -> MsSqlValue {
        MsSqlValue {
            data: self.data.cloned(),
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Option<Cow<'_, MsSqlTypeInfo>> {
        Some(Cow::Borrowed(&self.type_info))
    }

    fn is_null(&self) -> bool {
        self.data.is_none()
    }
}

/// Implementation of [`Value`] for MSSQL.
#[derive(Clone)]
pub struct MsSqlValue {
    pub(crate) type_info: MsSqlTypeInfo,
    pub(crate) data: Option<Bytes>,
}

impl Value for MsSqlValue {
    type Database = MsSql;

    fn as_ref(&self) -> MsSqlValueRef<'_> {
        MsSqlValueRef {
            data: self.data.as_ref(),
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Option<Cow<'_, MsSqlTypeInfo>> {
        Some(Cow::Borrowed(&self.type_info))
    }

    fn is_null(&self) -> bool {
        self.data.is_none()
    }
}
