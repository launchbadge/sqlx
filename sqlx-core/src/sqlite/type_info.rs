use std::fmt::{self, Display, Formatter};
use std::os::raw::c_int;
use std::str::FromStr;

use libsqlite3_sys::{SQLITE_BLOB, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT};

use crate::error::BoxDynError;
use crate::type_info::TypeInfo;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum DataType {
    Int,
    Float,
    Text,
    Blob,

    // TODO: Support NUMERIC
    #[allow(dead_code)]
    Numeric,

    // non-standard extensions
    Bool,
    Int64,
}

/// Type information for a SQLite type.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct SqliteTypeInfo(pub(crate) DataType);

impl Display for SqliteTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}

impl TypeInfo for SqliteTypeInfo {
    fn name(&self) -> &str {
        match self.0 {
            DataType::Text => "TEXT",
            DataType::Float => "FLOAT",
            DataType::Blob => "BLOB",
            DataType::Int => "INTEGER",
            DataType::Numeric => "NUMERIC",

            // non-standard extensions
            DataType::Bool => "BOOLEAN",
            DataType::Int64 => "BIGINT",
        }
    }
}

impl DataType {
    pub(crate) fn from_code(code: c_int) -> Option<Self> {
        match code {
            SQLITE_INTEGER => Some(DataType::Int),
            SQLITE_FLOAT => Some(DataType::Float),
            SQLITE_BLOB => Some(DataType::Blob),
            SQLITE_NULL => None,
            SQLITE_TEXT => Some(DataType::Text),

            _ => None,
        }
    }
}

// note: this implementation is particularly important as this is how the macros determine
//       what Rust type maps to what *declared* SQL type
// <https://www.sqlite.org/datatype3.html#affname>
impl FromStr for DataType {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        Ok(match &*s {
            "int8" => DataType::Int64,
            "boolean" | "bool" => DataType::Bool,

            _ if s.contains("int") && s.contains("big") && s.find("int") > s.find("big") => {
                DataType::Int64
            }

            _ if s.contains("int") => DataType::Int,

            _ if s.contains("char") || s.contains("clob") || s.contains("text") => DataType::Text,

            _ if s.contains("blob") => DataType::Blob,

            _ if s.contains("real") || s.contains("floa") || s.contains("doub") => DataType::Float,

            _ => {
                return Err(format!("unknown type: `{}`", s).into());
            }
        })
    }
}

#[test]
fn test_data_type_from_str() -> Result<(), BoxDynError> {
    assert_eq!(DataType::Int, "INT".parse()?);
    assert_eq!(DataType::Int, "INTEGER".parse()?);
    assert_eq!(DataType::Int, "INTBIG".parse()?);
    assert_eq!(DataType::Int, "MEDIUMINT".parse()?);

    assert_eq!(DataType::Int64, "BIGINT".parse()?);
    assert_eq!(DataType::Int64, "UNSIGNED BIG INT".parse()?);
    assert_eq!(DataType::Int64, "INT8".parse()?);

    assert_eq!(DataType::Text, "CHARACTER(20)".parse()?);
    assert_eq!(DataType::Text, "NCHAR(55)".parse()?);
    assert_eq!(DataType::Text, "TEXT".parse()?);
    assert_eq!(DataType::Text, "CLOB".parse()?);

    assert_eq!(DataType::Blob, "BLOB".parse()?);

    assert_eq!(DataType::Float, "REAL".parse()?);
    assert_eq!(DataType::Float, "FLOAT".parse()?);
    assert_eq!(DataType::Float, "DOUBLE PRECISION".parse()?);

    assert_eq!(DataType::Bool, "BOOLEAN".parse()?);
    assert_eq!(DataType::Bool, "BOOL".parse()?);

    Ok(())
}
