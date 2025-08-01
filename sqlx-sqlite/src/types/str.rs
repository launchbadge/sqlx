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

impl Encode<'_, Sqlite> for &'_ str {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self.to_string()));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for &'r str {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.text_borrowed()?)
    }
}

impl Encode<'_, Sqlite> for Box<str> {
    fn encode(self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self.into_string()));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self.to_string()));

        Ok(IsNull::No)
    }
}

impl Type<Sqlite> for String {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl Encode<'_, Sqlite> for String {
    fn encode(self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self.clone()));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for String {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.text_owned()?)
    }
}

impl Encode<'_, Sqlite> for Cow<'_, str> {
    fn encode(self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self.into()));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(self.to_string()));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Sqlite> for Arc<str> {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        <String as Encode<'_, Sqlite>>::encode(self.to_string(), args)
    }
}

impl Encode<'_, Sqlite> for Rc<str> {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue>) -> Result<IsNull, BoxDynError> {
        <String as Encode<'_, Sqlite>>::encode(self.to_string(), args)
    }
}
