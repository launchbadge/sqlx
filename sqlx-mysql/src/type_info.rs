use sqlx_core::TypeInfo;

use crate::protocol::{ColumnDefinition, ColumnFlags};
use crate::{MySql, MySqlTypeId};

/// Provides information about a MySQL type.
#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "offline", feature = "serde"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[allow(clippy::module_name_repetitions)]
pub struct MySqlTypeInfo {
    id: MySqlTypeId,
    charset: u16,
    flags: ColumnFlags,

    // [max_size] for integer types, this is (M) in BIT(M) or TINYINT(M)
    max_size: u32,
}

impl MySqlTypeInfo {
    pub(crate) fn new(def: &ColumnDefinition) -> Self {
        Self {
            id: MySqlTypeId::new(def),
            charset: def.charset,
            max_size: def.max_size,
            flags: def.flags,
        }
    }
}

impl MySqlTypeInfo {
    /// Returns the unique identifier for this MySQL type.
    #[must_use]
    pub const fn id(&self) -> MySqlTypeId {
        self.id
    }

    /// Returns `true` if this type has a binary collation.
    #[must_use]
    pub const fn has_binary_collation(&self) -> bool {
        self.flags.contains(ColumnFlags::BINARY_COLLATION)
    }

    /// Returns `true` if this type is `BOOLEAN`.
    #[must_use]
    pub const fn is_boolean(&self) -> bool {
        matches!(self.id(), MySqlTypeId::TINYINT | MySqlTypeId::TINYINT_UNSIGNED)
            && self.max_size == 1
    }

    /// Returns `true` if this type is an `ENUM`.
    #[must_use]
    pub const fn is_enum(&self) -> bool {
        self.flags.contains(ColumnFlags::ENUM)
    }

    /// Returns `true` if this type is a `SET`.
    #[must_use]
    pub const fn is_set(&self) -> bool {
        self.flags.contains(ColumnFlags::SET)
    }

    /// Returns the name for this MySQL type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self.id {
            MySqlTypeId::NULL => "NULL",

            // BOOLEAN has the same type ID as TINYINT
            _ if self.is_boolean() => "BOOLEAN",

            MySqlTypeId::TINYINT => "TINYINT",
            MySqlTypeId::SMALLINT => "SMALLINT",
            MySqlTypeId::MEDIUMINT => "MEDIUMINT",
            MySqlTypeId::INT => "INT",
            MySqlTypeId::BIGINT => "BIGINT",

            MySqlTypeId::TINYINT_UNSIGNED => "TINYINT UNSIGNED",
            MySqlTypeId::SMALLINT_UNSIGNED => "SMALLINT UNSIGNED",
            MySqlTypeId::MEDIUMINT_UNSIGNED => "MEDIUMINT UNSIGNED",
            MySqlTypeId::INT_UNSIGNED => "INT UNSIGNED",
            MySqlTypeId::BIGINT_UNSIGNED => "BIGINT UNSIGNED",

            MySqlTypeId::FLOAT => "FLOAT",
            MySqlTypeId::DOUBLE => "DOUBLE",

            // ENUM, and SET have the same type ID as CHAR
            _ if self.is_enum() => "ENUM",
            _ if self.is_set() => "SET",

            // VARBINARY, BINARY, and BLOB have the same type IDs as
            // VARCHAR, CHAR, and TEXT; the only difference is the
            // presence of a binary collation
            MySqlTypeId::VARCHAR if self.has_binary_collation() => "VARBINARY",
            MySqlTypeId::CHAR if self.has_binary_collation() => "BINARY",
            MySqlTypeId::TEXT if self.has_binary_collation() => "BLOB",

            MySqlTypeId::VARCHAR => "VARCHAR",
            MySqlTypeId::CHAR => "CHAR",
            MySqlTypeId::TEXT => "TEXT",

            _ => "UNKNOWN",
        }
    }
}

impl TypeInfo for MySqlTypeInfo {
    type Database = MySql;

    fn id(&self) -> MySqlTypeId {
        self.id()
    }

    fn is_unknown(&self) -> bool {
        self.id.is_null()
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

#[cfg(test)]
impl MySqlTypeInfo {
    pub(crate) const TINYINT_1: Self =
        Self { id: MySqlTypeId::TINYINT, max_size: 1, flags: ColumnFlags::empty(), charset: 0 };

    pub(crate) const BIGINT: Self =
        Self { id: MySqlTypeId::BIGINT, max_size: 0, flags: ColumnFlags::empty(), charset: 0 };

    pub(crate) const BINARY: Self = Self {
        id: MySqlTypeId::CHAR,
        max_size: 0,
        flags: ColumnFlags::BINARY_COLLATION,
        charset: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::MySqlTypeInfo;

    #[test]
    fn should_identify_boolean() {
        assert_eq!(MySqlTypeInfo::TINYINT_1.name(), "BOOLEAN");
    }

    #[test]
    fn should_identify_binary() {
        assert_eq!(MySqlTypeInfo::BINARY.name(), "BINARY");
    }
}
