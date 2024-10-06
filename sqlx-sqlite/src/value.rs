use std::borrow::Cow;
use std::ptr::NonNull;
use std::slice::from_raw_parts;
use std::str::from_utf8;
use std::sync::Arc;

use libsqlite3_sys::{
    sqlite3_value, sqlite3_value_blob, sqlite3_value_bytes, sqlite3_value_double,
    sqlite3_value_dup, sqlite3_value_free, sqlite3_value_int64, sqlite3_value_type, SQLITE_NULL,
};

pub(crate) use sqlx_core::value::{Value, ValueRef};

use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::{Sqlite, SqliteTypeInfo};

enum SqliteValueData<'r> {
    Value(&'r SqliteValue),
}

pub struct SqliteValueRef<'r>(SqliteValueData<'r>);

impl<'r> SqliteValueRef<'r> {
    pub(crate) fn value(value: &'r SqliteValue) -> Self {
        Self(SqliteValueData::Value(value))
    }

    // NOTE: `int()` is deliberately omitted because it will silently truncate a wider value,
    // which is likely to cause bugs:
    // https://github.com/launchbadge/sqlx/issues/3179
    // (Similar bug in Postgres): https://github.com/launchbadge/sqlx/issues/3161
    pub(super) fn int64(&self) -> i64 {
        match self.0 {
            SqliteValueData::Value(v) => v.int64(),
        }
    }

    pub(super) fn double(&self) -> f64 {
        match self.0 {
            SqliteValueData::Value(v) => v.double(),
        }
    }

    pub(super) fn blob(&self) -> &'r [u8] {
        match self.0 {
            SqliteValueData::Value(v) => v.blob(),
        }
    }

    pub(super) fn text(&self) -> Result<&'r str, BoxDynError> {
        match self.0 {
            SqliteValueData::Value(v) => v.text(),
        }
    }
}

impl<'r> ValueRef<'r> for SqliteValueRef<'r> {
    type Database = Sqlite;

    fn to_owned(&self) -> SqliteValue {
        match self.0 {
            SqliteValueData::Value(v) => v.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, SqliteTypeInfo> {
        match self.0 {
            SqliteValueData::Value(v) => v.type_info(),
        }
    }

    fn is_null(&self) -> bool {
        match self.0 {
            SqliteValueData::Value(v) => v.is_null(),
        }
    }
}

#[derive(Clone)]
pub struct SqliteValue {
    pub(crate) handle: Arc<ValueHandle>,
    pub(crate) type_info: SqliteTypeInfo,
}

pub(crate) struct ValueHandle(NonNull<sqlite3_value>);

// SAFE: only protected value objects are stored in SqliteValue
unsafe impl Send for ValueHandle {}
unsafe impl Sync for ValueHandle {}

impl SqliteValue {
    pub(crate) unsafe fn new(value: *mut sqlite3_value, type_info: SqliteTypeInfo) -> Self {
        debug_assert!(!value.is_null());

        Self {
            type_info,
            handle: Arc::new(ValueHandle(NonNull::new_unchecked(sqlite3_value_dup(
                value,
            )))),
        }
    }

    fn type_info_opt(&self) -> Option<SqliteTypeInfo> {
        let dt = DataType::from_code(unsafe { sqlite3_value_type(self.handle.0.as_ptr()) });

        if let DataType::Null = dt {
            None
        } else {
            Some(SqliteTypeInfo(dt))
        }
    }

    fn int64(&self) -> i64 {
        unsafe { sqlite3_value_int64(self.handle.0.as_ptr()) }
    }

    fn double(&self) -> f64 {
        unsafe { sqlite3_value_double(self.handle.0.as_ptr()) }
    }

    fn blob(&self) -> &[u8] {
        let len = unsafe { sqlite3_value_bytes(self.handle.0.as_ptr()) };

        // This likely means UB in SQLite itself or our usage of it;
        // signed integer overflow is UB in the C standard.
        let len = usize::try_from(len).unwrap_or_else(|_| {
            panic!("sqlite3_value_bytes() returned value out of range for usize: {len}")
        });

        if len == 0 {
            // empty blobs are NULL so just return an empty slice
            return &[];
        }

        let ptr = unsafe { sqlite3_value_blob(self.handle.0.as_ptr()) } as *const u8;
        debug_assert!(!ptr.is_null());

        unsafe { from_raw_parts(ptr, len) }
    }

    fn text(&self) -> Result<&str, BoxDynError> {
        Ok(from_utf8(self.blob())?)
    }
}

impl Value for SqliteValue {
    type Database = Sqlite;

    fn as_ref(&self) -> SqliteValueRef<'_> {
        SqliteValueRef::value(self)
    }

    fn type_info(&self) -> Cow<'_, SqliteTypeInfo> {
        self.type_info_opt()
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(&self.type_info))
    }

    fn is_null(&self) -> bool {
        unsafe { sqlite3_value_type(self.handle.0.as_ptr()) == SQLITE_NULL }
    }
}

impl Drop for ValueHandle {
    fn drop(&mut self) {
        unsafe {
            sqlite3_value_free(self.0.as_ptr());
        }
    }
}

// #[cfg(feature = "any")]
// impl<'r> From<SqliteValueRef<'r>> for crate::any::AnyValueRef<'r> {
//     #[inline]
//     fn from(value: SqliteValueRef<'r>) -> Self {
//         crate::any::AnyValueRef {
//             type_info: value.type_info().clone().into_owned().into(),
//             kind: crate::any::value::AnyValueRefKind::Sqlite(value),
//         }
//     }
// }
//
// #[cfg(feature = "any")]
// impl From<SqliteValue> for crate::any::AnyValue {
//     #[inline]
//     fn from(value: SqliteValue) -> Self {
//         crate::any::AnyValue {
//             type_info: value.type_info().clone().into_owned().into(),
//             kind: crate::any::value::AnyValueKind::Sqlite(value),
//         }
//     }
// }
