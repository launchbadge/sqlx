use crate::arguments::SqliteArgumentsBuffer;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use std::sync::Arc;
use url::Url;

impl Type<Sqlite> for Url {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Text)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Text)
    }
}

impl Encode<'_, Sqlite> for Url {
    fn encode_by_ref(&self, args: &mut SqliteArgumentsBuffer) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Text(Arc::new(
            self.as_str().to_string(),
        )));

        Ok(IsNull::No)
    }
}

impl Decode<'_, Sqlite> for Url {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        Url::parse(value.text_borrowed()?).map_err(Into::into)
    }
}
