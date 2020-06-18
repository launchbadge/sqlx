use std::fmt::{self, Display, Formatter};

use crate::mysql::protocol::text::{ColumnDefinition, ColumnFlags, ColumnType};
use crate::type_info::TypeInfo;

/// Type information for a MySql type.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MySqlTypeInfo {
    pub(crate) r#type: ColumnType,
    pub(crate) flags: ColumnFlags,
    pub(crate) char_set: u16,
}

impl MySqlTypeInfo {
    pub(crate) const fn binary(ty: ColumnType) -> Self {
        Self {
            r#type: ty,
            flags: ColumnFlags::BINARY,
            char_set: 63,
        }
    }

    #[doc(hidden)]
    pub const fn __enum() -> Self {
        Self {
            r#type: ColumnType::Enum,
            flags: ColumnFlags::BINARY,
            char_set: 63,
        }
    }

    #[doc(hidden)]
    pub fn __type_feature_gate(&self) -> Option<&'static str> {
        match self.r#type {
            ColumnType::Date | ColumnType::Time | ColumnType::Timestamp | ColumnType::Datetime => {
                Some("time")
            }

            ColumnType::Json => Some("json"),
            ColumnType::NewDecimal => Some("bigdecimal"),

            _ => None,
        }
    }

    pub(crate) fn from_column(column: &ColumnDefinition) -> Option<Self> {
        if column.r#type == ColumnType::Null {
            None
        } else {
            Some(Self {
                r#type: column.r#type,
                flags: column.flags,
                char_set: column.char_set,
            })
        }
    }
}

impl Display for MySqlTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}

impl TypeInfo for MySqlTypeInfo {
    fn name(&self) -> &str {
        self.r#type.name(self.char_set, self.flags)
    }
}

impl PartialEq<MySqlTypeInfo> for MySqlTypeInfo {
    fn eq(&self, other: &MySqlTypeInfo) -> bool {
        if self.r#type != other.r#type {
            return false;
        }

        match self.r#type {
            ColumnType::Tiny
            | ColumnType::Short
            | ColumnType::Long
            | ColumnType::Int24
            | ColumnType::LongLong => {
                return self.flags.contains(ColumnFlags::UNSIGNED)
                    == other.flags.contains(ColumnFlags::UNSIGNED);
            }

            // for string types, check that our charset matches
            ColumnType::VarChar
            | ColumnType::Blob
            | ColumnType::TinyBlob
            | ColumnType::MediumBlob
            | ColumnType::LongBlob
            | ColumnType::String
            | ColumnType::VarString
            | ColumnType::Enum => {
                return self.char_set == other.char_set;
            }

            _ => {}
        }

        true
    }
}

impl Eq for MySqlTypeInfo {}
