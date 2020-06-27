use std::borrow::Cow;

use crate::any::type_info::AnyTypeInfoKind;
use crate::any::{Any, AnyTypeInfo};
use crate::database::HasValueRef;
use crate::value::{Value, ValueRef};

#[cfg(feature = "postgres")]
use crate::postgres::{PgValue, PgValueRef};

#[cfg(feature = "mysql")]
use crate::mysql::{MySqlValue, MySqlValueRef};

#[cfg(feature = "sqlite")]
use crate::sqlite::{SqliteValue, SqliteValueRef};

#[cfg(feature = "mssql")]
use crate::mssql::{MssqlValue, MssqlValueRef};

pub struct AnyValue(AnyValueKind);

pub(crate) enum AnyValueKind {
    #[cfg(feature = "postgres")]
    Postgres(PgValue),

    #[cfg(feature = "mysql")]
    MySql(MySqlValue),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteValue),

    #[cfg(feature = "mssql")]
    Mssql(MssqlValue),
}

pub struct AnyValueRef<'r>(pub(crate) AnyValueRefKind<'r>);

pub(crate) enum AnyValueRefKind<'r> {
    #[cfg(feature = "postgres")]
    Postgres(PgValueRef<'r>),

    #[cfg(feature = "mysql")]
    MySql(MySqlValueRef<'r>),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteValueRef<'r>),

    #[cfg(feature = "mssql")]
    Mssql(MssqlValueRef<'r>),
}

impl Value for AnyValue {
    type Database = Any;

    fn as_ref(&self) -> <Self::Database as HasValueRef<'_>>::ValueRef {
        AnyValueRef(match &self.0 {
            #[cfg(feature = "postgres")]
            AnyValueKind::Postgres(value) => AnyValueRefKind::Postgres(value.as_ref()),

            #[cfg(feature = "mysql")]
            AnyValueKind::MySql(value) => AnyValueRefKind::MySql(value.as_ref()),

            #[cfg(feature = "sqlite")]
            AnyValueKind::Sqlite(value) => AnyValueRefKind::Sqlite(value.as_ref()),

            #[cfg(feature = "mssql")]
            AnyValueKind::Mssql(value) => AnyValueRefKind::Mssql(value.as_ref()),
        })
    }

    fn type_info(&self) -> Option<Cow<'_, AnyTypeInfo>> {
        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyValueKind::Postgres(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::Postgres(ty.into_owned())),

            #[cfg(feature = "mysql")]
            AnyValueKind::MySql(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::MySql(ty.into_owned())),

            #[cfg(feature = "sqlite")]
            AnyValueKind::Sqlite(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::Sqlite(ty.into_owned())),

            #[cfg(feature = "mssql")]
            AnyValueKind::Mssql(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::Mssql(ty.into_owned())),
        }
        .map(AnyTypeInfo)
        .map(Cow::Owned)
    }

    fn is_null(&self) -> bool {
        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyValueKind::Postgres(value) => value.is_null(),

            #[cfg(feature = "mysql")]
            AnyValueKind::MySql(value) => value.is_null(),

            #[cfg(feature = "sqlite")]
            AnyValueKind::Sqlite(value) => value.is_null(),

            #[cfg(feature = "mssql")]
            AnyValueKind::Mssql(value) => value.is_null(),
        }
    }
}

impl<'r> ValueRef<'r> for AnyValueRef<'r> {
    type Database = Any;

    fn to_owned(&self) -> AnyValue {
        AnyValue(match &self.0 {
            #[cfg(feature = "postgres")]
            AnyValueRefKind::Postgres(value) => AnyValueKind::Postgres(ValueRef::to_owned(value)),

            #[cfg(feature = "mysql")]
            AnyValueRefKind::MySql(value) => AnyValueKind::MySql(ValueRef::to_owned(value)),

            #[cfg(feature = "sqlite")]
            AnyValueRefKind::Sqlite(value) => AnyValueKind::Sqlite(ValueRef::to_owned(value)),

            #[cfg(feature = "mssql")]
            AnyValueRefKind::Mssql(value) => AnyValueKind::Mssql(ValueRef::to_owned(value)),
        })
    }

    fn type_info(&self) -> Option<Cow<'_, AnyTypeInfo>> {
        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyValueRefKind::Postgres(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::Postgres(ty.into_owned())),

            #[cfg(feature = "mysql")]
            AnyValueRefKind::MySql(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::MySql(ty.into_owned())),

            #[cfg(feature = "sqlite")]
            AnyValueRefKind::Sqlite(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::Sqlite(ty.into_owned())),

            #[cfg(feature = "mssql")]
            AnyValueRefKind::Mssql(value) => value
                .type_info()
                .map(|ty| AnyTypeInfoKind::Mssql(ty.into_owned())),
        }
        .map(AnyTypeInfo)
        .map(Cow::Owned)
    }

    fn is_null(&self) -> bool {
        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyValueRefKind::Postgres(value) => value.is_null(),

            #[cfg(feature = "mysql")]
            AnyValueRefKind::MySql(value) => value.is_null(),

            #[cfg(feature = "sqlite")]
            AnyValueRefKind::Sqlite(value) => value.is_null(),

            #[cfg(feature = "mssql")]
            AnyValueRefKind::Mssql(value) => value.is_null(),
        }
    }
}
