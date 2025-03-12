use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use sqlx_core::database::Database;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};

impl Type<Sqlite> for str {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Text)
    }
}

impl<'q> Encode<'q, Sqlite> for &'q str {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Cow::Borrowed(*self)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for &'r str {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        value.text()
    }
}

impl Type<Sqlite> for String {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for String {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        value.text().map(ToOwned::to_owned)
    }
}

impl<'q> Encode<'q, Sqlite> for String {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Cow::Owned(self.clone())));

        Ok(IsNull::No)
    }

    fn encode(
        self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError>
    where
        Self: Sized,
    {
        buf.push(SqliteArgumentValue::Text(Cow::Owned(self)));

        Ok(IsNull::No)
    }
}

forward_encode_impl!(Arc<str>, String, Sqlite, |s: &str| s.to_string());
forward_encode_impl!(Rc<str>, String, Sqlite, |s: &str| s.to_string());
forward_encode_impl!(Cow<'_, str>, String, Sqlite, |s: &str| s.to_string());
forward_encode_impl!(Box<str>, String, Sqlite, |s: &str| s.to_string());
