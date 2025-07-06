use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

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

impl Encode<'_, Sqlite> for Box<str> {
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Cow::Owned(self.into_string())));

        Ok(IsNull::No)
    }

    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Cow::Owned(
            self.clone().into_string(),
        )));

        Ok(IsNull::No)
    }
}

impl Type<Sqlite> for String {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for String {
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Cow::Owned(self)));

        Ok(IsNull::No)
    }

    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Cow::Owned(self.clone())));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for String {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        value.text().map(ToOwned::to_owned)
    }
}

impl<'q> Encode<'q, Sqlite> for Cow<'q, str> {
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self));

        Ok(IsNull::No)
    }

    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self.clone()));

        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Sqlite> for Arc<str> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<'_, Sqlite>>::encode(self.to_string(), args)
    }
}

impl<'q> Encode<'q, Sqlite> for Rc<str> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<'_, Sqlite>>::encode(self.to_string(), args)
    }
}
