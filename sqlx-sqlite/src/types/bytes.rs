use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};

impl Type<Sqlite> for [u8] {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Blob)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Blob | DataType::Text)
    }
}

impl<'q> Encode<'q, Sqlite> for &'q [u8] {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(Cow::Borrowed(self)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for &'r [u8] {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.blob_borrowed())
    }
}

impl Encode<'_, Sqlite> for Box<[u8]> {
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(Cow::Owned(self.into_vec())));

        Ok(IsNull::No)
    }

    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(Cow::Owned(
            self.clone().into_vec(),
        )));

        Ok(IsNull::No)
    }
}

impl Type<Sqlite> for Vec<u8> {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&[u8] as Type<Sqlite>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Sqlite> for Vec<u8> {
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(Cow::Owned(self)));

        Ok(IsNull::No)
    }

    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(Cow::Owned(self.clone())));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for Vec<u8> {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.blob_owned())
    }
}

impl<'q> Encode<'q, Sqlite> for Cow<'q, [u8]> {
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(self));

        Ok(IsNull::No)
    }

    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(self.clone()));

        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Sqlite> for Arc<[u8]> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        <Vec<u8> as Encode<'_, Sqlite>>::encode(self.to_vec(), args)
    }
}

impl<'q> Encode<'q, Sqlite> for Rc<[u8]> {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        <Vec<u8> as Encode<'_, Sqlite>>::encode(self.to_vec(), args)
    }
}
