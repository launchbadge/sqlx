use bytestring::ByteString;
use sqlx_core::Column;

use crate::protocol::{ColumnDefinition, ColumnFlags};
use crate::{MySql, MySqlTypeInfo};

/// Represents a column from a query in MySQL.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct MySqlColumn {
    index: usize,
    name: ByteString,
    type_info: MySqlTypeInfo,
    flags: ColumnFlags,
}

impl MySqlColumn {
    pub(crate) fn new(index: usize, def: ColumnDefinition) -> Self {
        let type_info = MySqlTypeInfo::new(&def);

        // use either the column alias or name
        // prefer alias if its non-empty
        let name = if def.alias.is_empty() { def.name } else { def.alias };

        Self { type_info, index, name, flags: def.flags }
    }
}

impl MySqlColumn {
    /// Returns the name of the column.
    ///
    /// If the original name of the column has been aliased, this will return
    /// the aliased name.
    ///
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns type information of the column.
    pub fn type_info(&self) -> &MySqlTypeInfo {
        &self.type_info
    }

    /// Returns the (zero-based) position of the column.
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns `true` if the column is (or is part of) a `PRIMARY KEY`.
    pub const fn is_primary_key(&self) -> bool {
        self.flags.contains(ColumnFlags::PRIMARY_KEY)
    }

    /// Returns `true` if the column is nullable.
    pub const fn is_nullable(&self) -> bool {
        !self.flags.contains(ColumnFlags::NOT_NULL)
    }

    /// Returns `true` if the column is has a default value.
    ///
    /// Always returns `true` if the column is nullable as `NULL` is
    /// a valid default value.
    ///
    pub const fn has_default_value(&self) -> bool {
        !self.flags.contains(ColumnFlags::NO_DEFAULT_VALUE)
    }
}

impl Column for MySqlColumn {
    type Database = MySql;

    #[inline]
    fn name(&self) -> &str {
        self.name()
    }

    #[inline]
    fn index(&self) -> usize {
        self.index()
    }

    #[inline]
    fn type_info(&self) -> &MySqlTypeInfo {
        self.type_info()
    }
}
