use sqlx_core::TypeInfo;

use crate::protocol::ColumnDefinition;
use crate::{MySql, MySqlTypeId};

/// Provides information about a MySQL type.
#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "offline", feature = "serde"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MySqlTypeInfo {
    id: MySqlTypeId,
    charset: u16,

    // [max_size] for integer types, this is (M) in BIT(M) or TINYINT(M)
    max_size: u32,
}

impl MySqlTypeInfo {
    pub(crate) const fn new(def: &ColumnDefinition) -> Self {
        Self { id: MySqlTypeId::new(def), charset: def.charset, max_size: def.max_size }
    }
}

impl MySqlTypeInfo {
    /// Returns the unique identifier for this MySQL type.
    pub const fn id(&self) -> MySqlTypeId {
        self.id
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
        self.id.name()
    }
}
