use std::fmt::{self, Display};

use crate::decode::Decode;
use crate::sqlite::value::SqliteValue;
use crate::sqlite::Sqlite;
use crate::types::TypeInfo;

mod bool;
mod bytes;
mod float;
mod int;
mod str;

// https://www.sqlite.org/c3ref/c_blob.html
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum SqliteType {
    Integer = 1,
    Float = 2,
    Text = 3,
    Blob = 4,
    Null = 5,

    // Non-standard extensions
    Boolean,
}

// https://www.sqlite.org/datatype3.html#type_affinity
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum SqliteTypeAffinity {
    Text,
    Numeric,
    Integer,
    Real,
    Blob,
}

#[derive(Debug, Clone)]
pub struct SqliteTypeInfo {
    pub(crate) r#type: SqliteType,
    pub(crate) affinity: Option<SqliteTypeAffinity>,
}

impl SqliteTypeInfo {
    fn new(r#type: SqliteType, affinity: SqliteTypeAffinity) -> Self {
        Self {
            r#type,
            affinity: Some(affinity),
        }
    }
}

impl Display for SqliteTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.r#type {
            SqliteType::Null => "NULL",
            SqliteType::Text => "TEXT",
            SqliteType::Boolean => "BOOLEAN",
            SqliteType::Integer => "INTEGER",
            SqliteType::Float => "DOUBLE",
            SqliteType::Blob => "BLOB",
        })
    }
}

impl TypeInfo for SqliteTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        self.affinity == other.affinity
    }

    fn is_null_type(&self) -> bool {
        self.r#type == SqliteType::Null
    }
}

impl<'de, T> Decode<'de, Sqlite> for Option<T>
where
    T: Decode<'de, Sqlite>,
{
    fn decode(value: SqliteValue<'de>) -> crate::Result<Self> {
        match value.r#type() {
            SqliteType::Null => Ok(None),
            _ => <T as Decode<Sqlite>>::decode(value).map(Some),
        }
    }
}
