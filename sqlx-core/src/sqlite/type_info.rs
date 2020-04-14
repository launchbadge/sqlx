use std::fmt::{self, Display};

use crate::types::TypeInfo;

// https://www.sqlite.org/c3ref/c_blob.html
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum SqliteType {
    Integer = 1,
    Float = 2,
    Text = 3,
    Blob = 4,

    // Non-standard extensions
    Boolean,
}

// https://www.sqlite.org/datatype3.html#type_affinity
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum SqliteTypeAffinity {
    Text,
    Numeric,
    Integer,
    Real,
    Blob,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct SqliteTypeInfo {
    pub(crate) r#type: SqliteType,
    pub(crate) affinity: Option<SqliteTypeAffinity>,
}

impl SqliteTypeInfo {
    pub(crate) fn new(r#type: SqliteType, affinity: SqliteTypeAffinity) -> Self {
        Self {
            r#type,
            affinity: Some(affinity),
        }
    }
}

impl Display for SqliteTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.r#type {
            SqliteType::Text => "TEXT",
            SqliteType::Boolean => "BOOLEAN",
            SqliteType::Integer => "INTEGER",
            SqliteType::Float => "DOUBLE",
            SqliteType::Blob => "BLOB",
        })
    }
}

impl PartialEq<SqliteTypeInfo> for SqliteTypeInfo {
    fn eq(&self, other: &SqliteTypeInfo) -> bool {
        self.r#type == other.r#type || self.affinity == other.affinity
    }
}

impl TypeInfo for SqliteTypeInfo {
    #[inline]
    fn compatible(&self, _other: &Self) -> bool {
        // All types are compatible with all other types in SQLite
        true
    }
}
