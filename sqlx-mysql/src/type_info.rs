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
    has_binary_collation: bool,

    // [max_size] for integer types, this is (M) in BIT(M) or TINYINT(M)
    max_size: u32,
}

impl MySqlTypeInfo {
    pub(crate) const fn new(def: &ColumnDefinition) -> Self {
        Self {
            id: MySqlTypeId::new(def),
            charset: def.charset,
            max_size: def.max_size,
            has_binary_collation: def.flags.contains(ColumnFlags::BINARY_COLLATION),
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
        self.has_binary_collation
    }

    /// Returns the name for this MySQL type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self.id {
            MySqlTypeId::NULL => "NULL",

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

            // note: VARBINARY, BINARY, and BLOB have the same type IDs as
            //       VARCHAR, CHAR, and TEXT; the only difference is the
            //       presence of a binary collation
            MySqlTypeId::VARBINARY if self.has_binary_collation() => "VARBINARY",
            MySqlTypeId::BINARY if self.has_binary_collation() => "BINARY",
            MySqlTypeId::BLOB if self.has_binary_collation() => "BLOB",

            MySqlTypeId::VARCHAR => "VARCHAR",
            MySqlTypeId::CHAR => "CHAR",
            MySqlTypeId::TEXT => "TEXT",

            _ => "",
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

    fn name(&self) -> &str {
        self.name()
    }
}
