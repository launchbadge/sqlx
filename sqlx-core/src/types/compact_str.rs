#[cfg(feature = "sqlite")]
use std::borrow::Cow;

/// Conversions between `bstr` types and SQL types.
use crate::database::{Database, HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;

#[cfg(all(
    any(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite"
    ),
    feature = "any"
))]
use crate::any::{Any, AnyArgumentBuffer, AnyArgumentBufferKind, AnyEncode};
#[cfg(feature = "mssql")]
use crate::mssql::Mssql;
#[cfg(feature = "mysql")]
use crate::mysql::MySql;
#[cfg(feature = "postgres")]
use crate::postgres::Postgres;
#[cfg(feature = "sqlite")]
use crate::sqlite::{Sqlite, SqliteArgumentValue};

#[doc(no_inline)]
pub use compact_str_::CompactString;

impl<DB> Type<DB> for CompactString
where
    DB: Database,
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB> Decode<'r, DB> for CompactString
where
    DB: Database,
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        <&str as Decode<DB>>::decode(value).map(CompactString::new)
    }
}

#[cfg(feature = "postgres")]
impl<'q> Encode<'q, Postgres> for CompactString {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

#[cfg(feature = "mysql")]
impl<'q> Encode<'q, MySql> for CompactString {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <&str as Encode<MySql>>::encode(self.as_str(), buf)
    }
}

#[cfg(feature = "mssql")]
impl<'q> Encode<'q, Mssql> for CompactString {
    fn encode_by_ref(&self, buf: &mut <Mssql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <&str as Encode<Mssql>>::encode(self.as_str(), buf)
    }
}

#[cfg(feature = "sqlite")]
impl<'q> Encode<'q, Sqlite> for CompactString {
    fn encode_by_ref(&self, buf: &mut <Sqlite as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.push(SqliteArgumentValue::Blob(Cow::Owned(
            self.as_bytes().to_vec(),
        )));
        IsNull::No
    }
}

#[cfg(all(
    any(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite"
    ),
    feature = "any"
))]
impl<'q> Encode<'q, Any> for CompactString
where
    CompactString: AnyEncode<'q>,
{
    fn encode_by_ref(&self, buf: &mut AnyArgumentBuffer<'q>) -> IsNull {
        match &mut buf.0 {
            #[cfg(feature = "postgres")]
            AnyArgumentBufferKind::Postgres(args, _) => args.add(self),
            #[cfg(feature = "mysql")]
            AnyArgumentBufferKind::MySql(args, _) => args.add(self),
            #[cfg(feature = "mssql")]
            AnyArgumentBufferKind::Mssql(args, _) => args.add(self),
            #[cfg(feature = "sqlite")]
            AnyArgumentBufferKind::Sqlite(args) => args.add(self),
        }
        IsNull::No
    }
}
