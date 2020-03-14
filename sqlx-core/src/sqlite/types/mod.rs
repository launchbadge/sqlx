use std::fmt::{self, Display};

use crate::decode::Decode;
use crate::sqlite::value::SqliteResultValue;
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
    Blob = 4,
    Null = 5,
    Text = 3,
}

// https://www.sqlite.org/datatype3.html#type_affinity
#[derive(Debug, PartialEq, Clone, Copy)]
enum SqliteTypeAffinity {
    Text,
    Numeric,
    Integer,
    Real,
    Blob,
}

#[derive(Debug, Clone)]
pub struct SqliteTypeInfo {
    r#type: SqliteType,
    affinity: SqliteTypeAffinity,
}

impl SqliteTypeInfo {
    fn new(r#type: SqliteType, affinity: SqliteTypeAffinity) -> Self {
        Self { r#type, affinity }
    }
}

impl Display for SqliteTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.affinity {
            SqliteTypeAffinity::Text => "TEXT",
            SqliteTypeAffinity::Numeric => "NUMERIC",
            SqliteTypeAffinity::Integer => "INTEGER",
            SqliteTypeAffinity::Real => "REAL",
            SqliteTypeAffinity::Blob => "BLOB",
        })
    }
}

impl TypeInfo for SqliteTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        self.affinity == other.affinity
    }
}

impl<'de, T> Decode<'de, Sqlite> for Option<T>
where
    T: Decode<'de, Sqlite>,
{
    fn decode(value: SqliteResultValue<'de>) -> crate::Result<Self> {
        match value.r#type() {
            SqliteType::Null => Ok(None),
            _ => <T as Decode<Sqlite>>::decode(value).map(Some),
        }
    }
}
