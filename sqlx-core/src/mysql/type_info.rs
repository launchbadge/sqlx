use std::fmt::{self, Display};

use crate::mysql::protocol::{ColumnDefinition, FieldFlags, TypeId};
use crate::types::TypeInfo;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MySqlTypeInfo {
    pub(crate) id: TypeId,
    pub(crate) is_unsigned: bool,
    pub(crate) is_binary: bool,
    pub(crate) char_set: u16,
}

impl MySqlTypeInfo {
    pub(crate) const fn new(id: TypeId) -> Self {
        Self {
            id,
            is_unsigned: false,
            is_binary: true,
            char_set: 0,
        }
    }

    pub(crate) const fn unsigned(id: TypeId) -> Self {
        Self {
            id,
            is_unsigned: true,
            is_binary: false,
            char_set: 0,
        }
    }

    #[doc(hidden)]
    pub const fn r#enum() -> Self {
        Self {
            id: TypeId::ENUM,
            is_unsigned: false,
            is_binary: false,
            char_set: 0,
        }
    }

    pub(crate) fn from_nullable_column_def(def: &ColumnDefinition) -> Self {
        Self {
            id: def.type_id,
            is_unsigned: def.flags.contains(FieldFlags::UNSIGNED),
            is_binary: def.flags.contains(FieldFlags::BINARY),
            char_set: def.char_set,
        }
    }

    pub(crate) fn from_column_def(def: &ColumnDefinition) -> Option<Self> {
        if def.type_id == TypeId::NULL {
            return None;
        }

        Some(Self::from_nullable_column_def(def))
    }

    #[doc(hidden)]
    pub fn type_feature_gate(&self) -> Option<&'static str> {
        match self.id {
            TypeId::DATE | TypeId::TIME | TypeId::DATETIME | TypeId::TIMESTAMP => Some("chrono"),
            _ => None,
        }
    }
}

impl Display for MySqlTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            TypeId::NULL => f.write_str("NULL"),

            TypeId::TINY_INT if self.is_unsigned => f.write_str("TINYINT UNSIGNED"),
            TypeId::SMALL_INT if self.is_unsigned => f.write_str("SMALLINT UNSIGNED"),
            TypeId::INT if self.is_unsigned => f.write_str("INT UNSIGNED"),
            TypeId::BIG_INT if self.is_unsigned => f.write_str("BIGINT UNSIGNED"),

            TypeId::TINY_INT => f.write_str("TINYINT"),
            TypeId::SMALL_INT => f.write_str("SMALLINT"),
            TypeId::INT => f.write_str("INT"),
            TypeId::BIG_INT => f.write_str("BIGINT"),

            TypeId::FLOAT => f.write_str("FLOAT"),
            TypeId::DOUBLE => f.write_str("DOUBLE"),

            TypeId::CHAR if self.is_binary => f.write_str("BINARY"),
            TypeId::VAR_CHAR if self.is_binary => f.write_str("VARBINARY"),
            TypeId::TEXT if self.is_binary => f.write_str("BLOB"),

            TypeId::CHAR => f.write_str("CHAR"),
            TypeId::VAR_CHAR => f.write_str("VARCHAR"),
            TypeId::TEXT => f.write_str("TEXT"),

            TypeId::DATE => f.write_str("DATE"),
            TypeId::TIME => f.write_str("TIME"),
            TypeId::DATETIME => f.write_str("DATETIME"),
            TypeId::TIMESTAMP => f.write_str("TIMESTAMP"),

            id => write!(f, "<{:#x}>", id.0),
        }
    }
}

impl PartialEq<MySqlTypeInfo> for MySqlTypeInfo {
    fn eq(&self, other: &MySqlTypeInfo) -> bool {
        match self.id {
            TypeId::VAR_CHAR
            | TypeId::TEXT
            | TypeId::CHAR
            | TypeId::TINY_BLOB
            | TypeId::MEDIUM_BLOB
            | TypeId::LONG_BLOB
            | TypeId::ENUM
                if (self.is_binary == other.is_binary)
                    && match other.id {
                        TypeId::VAR_CHAR
                        | TypeId::TEXT
                        | TypeId::CHAR
                        | TypeId::TINY_BLOB
                        | TypeId::MEDIUM_BLOB
                        | TypeId::LONG_BLOB
                        | TypeId::ENUM => true,

                        _ => false,
                    } =>
            {
                return true;
            }

            _ => {}
        }

        if self.id.0 != other.id.0 {
            return false;
        }

        match self.id {
            TypeId::TINY_INT | TypeId::SMALL_INT | TypeId::INT | TypeId::BIG_INT => {
                return self.is_unsigned == other.is_unsigned;
            }

            _ => {}
        }

        true
    }
}

impl TypeInfo for MySqlTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        // NOTE: MySQL is weakly typed so much of this may be surprising to a Rust developer.

        if self.id == TypeId::NULL || other.id == TypeId::NULL {
            // NULL is the "bottom" type
            // If the user is trying to select into a non-Option, we catch this soon with an
            // UnexpectedNull error message
            return true;
        }

        match self.id {
            // All integer types should be considered compatible
            TypeId::TINY_INT | TypeId::SMALL_INT | TypeId::INT | TypeId::BIG_INT
                if (self.is_unsigned == other.is_unsigned)
                    && match other.id {
                        TypeId::TINY_INT | TypeId::SMALL_INT | TypeId::INT | TypeId::BIG_INT => {
                            true
                        }

                        _ => false,
                    } =>
            {
                true
            }

            // All textual types should be considered compatible
            TypeId::VAR_CHAR
            | TypeId::TEXT
            | TypeId::CHAR
            | TypeId::TINY_BLOB
            | TypeId::MEDIUM_BLOB
            | TypeId::LONG_BLOB
                if match other.id {
                    TypeId::VAR_CHAR
                    | TypeId::TEXT
                    | TypeId::CHAR
                    | TypeId::TINY_BLOB
                    | TypeId::MEDIUM_BLOB
                    | TypeId::LONG_BLOB => true,

                    _ => false,
                } =>
            {
                true
            }

            // Enums are considered compatible with other text/binary types
            TypeId::ENUM
                if match other.id {
                    TypeId::VAR_CHAR
                    | TypeId::TEXT
                    | TypeId::CHAR
                    | TypeId::TINY_BLOB
                    | TypeId::MEDIUM_BLOB
                    | TypeId::LONG_BLOB
                    | TypeId::ENUM => true,

                    _ => false,
                } =>
            {
                true
            }

            TypeId::VAR_CHAR
            | TypeId::TEXT
            | TypeId::CHAR
            | TypeId::TINY_BLOB
            | TypeId::MEDIUM_BLOB
            | TypeId::LONG_BLOB
            | TypeId::ENUM
                if other.id == TypeId::ENUM =>
            {
                true
            }

            // FLOAT is compatible with DOUBLE
            TypeId::FLOAT | TypeId::DOUBLE
                if match other.id {
                    TypeId::FLOAT | TypeId::DOUBLE => true,
                    _ => false,
                } =>
            {
                true
            }

            // DATETIME is compatible with TIMESTAMP
            TypeId::DATETIME | TypeId::TIMESTAMP
                if match other.id {
                    TypeId::DATETIME | TypeId::TIMESTAMP => true,
                    _ => false,
                } =>
            {
                true
            }

            _ => self.eq(other),
        }
    }
}
