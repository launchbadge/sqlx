use crate::error::UnexpectedNullError;
use crate::mysql::{MySql, MySqlTypeInfo};
use crate::value::RawValue;

#[derive(Debug, Copy, Clone)]
pub enum MySqlData<'c> {
    Binary(&'c [u8]),
    Text(&'c [u8]),
}

#[derive(Debug)]
pub struct MySqlValue<'c> {
    type_info: Option<MySqlTypeInfo>,
    data: Option<MySqlData<'c>>,
}

impl<'c> MySqlValue<'c> {
    /// Gets the binary or text data for this value; or, `UnexpectedNullError` if this
    /// is a `NULL` value.
    pub(crate) fn try_get(&self) -> crate::Result<MySql, MySqlData<'c>> {
        match self.data {
            Some(data) => Ok(data),
            None => Err(crate::Error::decode(UnexpectedNullError)),
        }
    }

    /// Gets the binary or text data for this value; or, `None` if this
    /// is a `NULL` value.
    #[inline]
    pub fn get(&self) -> Option<MySqlData<'c>> {
        self.data
    }

    pub(crate) fn null() -> Self {
        Self {
            type_info: None,
            data: None,
        }
    }

    pub(crate) fn binary(type_info: MySqlTypeInfo, buf: &'c [u8]) -> Self {
        Self {
            type_info: Some(type_info),
            data: Some(MySqlData::Binary(buf)),
        }
    }

    pub(crate) fn text(type_info: MySqlTypeInfo, buf: &'c [u8]) -> Self {
        Self {
            type_info: Some(type_info),
            data: Some(MySqlData::Text(buf)),
        }
    }
}

impl<'c> RawValue<'c> for MySqlValue<'c> {
    type Database = MySql;

    fn type_info(&self) -> Option<MySqlTypeInfo> {
        self.type_info.clone()
    }
}
