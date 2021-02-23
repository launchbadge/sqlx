use crate::protocol::{ColumnDefinition, ColumnFlags};

/// A unique identifier for a MySQL data type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    any(feature = "offline", feature = "serde"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[allow(clippy::module_name_repetitions)]
pub struct MySqlTypeId(u8, u8);

// a flag byte which has the highest bit set to indicate the type is unsigned
const UNSIGNED: u8 = 0x80;

impl MySqlTypeId {
    pub(crate) const fn new(def: &ColumnDefinition) -> Self {
        Self(def.ty, if def.flags.contains(ColumnFlags::UNSIGNED) { UNSIGNED } else { 0 })
    }

    pub(crate) const fn ty(self) -> u8 {
        self.0
    }

    pub(crate) const fn flags(self) -> u8 {
        self.1
    }
}

impl MySqlTypeId {
    /// Returns `true` if this is the `NULL` type.
    ///
    /// For MySQL, this occurs in types from parameters or when `NULL` is
    /// directly used in an expression by itself, such as `SELECT NULL`.
    ///
    pub(crate) const fn is_null(&self) -> bool {
        matches!(*self, MySqlTypeId::NULL)
    }

    /// Returns `true` if this is an integer data type.
    pub(crate) const fn is_integer(&self) -> bool {
        matches!(
            *self,
            Self::TINYINT
                | Self::TINYINT_UNSIGNED
                | Self::SMALLINT
                | Self::SMALLINT_UNSIGNED
                | Self::MEDIUMINT
                | Self::MEDIUMINT_UNSIGNED
                | Self::INT
                | Self::INT_UNSIGNED
                | Self::BIGINT
                | Self::BIGINT_UNSIGNED
        )
    }

    /// Returns `true` if this is an unsigned data type.
    pub(crate) const fn is_unsigned(&self) -> bool {
        self.1 == UNSIGNED
    }
}

// https://dev.mysql.com/doc/refman/8.0/en/data-types.html
// https://github.com/mysql/mysql-server/blob/7ed30a748964c009d4909cb8b4b22036ebdef239/include/field_types.h#L57

impl MySqlTypeId {
    /// An 8-bit integer.
    ///
    /// Compatible with any primitive integer type or `bool`.
    ///
    /// If the display width is 1 ( `TINYINT(1)` ), maps to `bool`. Otherwise,
    /// maps to `i8`.
    ///
    pub const TINYINT: Self = Self(1, 0);

    /// An unsigned 8-bit integer.
    ///
    /// Compatible with any primitive integer type or `bool`.
    ///
    /// If the display width is 1 ( `TINYINT(1) UNSIGNED` ), maps to `bool`. Otherwise,
    /// maps to `u8`.
    ///
    pub const TINYINT_UNSIGNED: Self = Self(1, UNSIGNED);

    /// A 16-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i16`.
    ///
    pub const SMALLINT: Self = Self(2, 0);

    /// An unsigned 16-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `u16`.
    ///
    pub const SMALLINT_UNSIGNED: Self = Self(2, UNSIGNED);

    /// A 24-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i32`.
    ///
    pub const MEDIUMINT: Self = Self(9, 0);

    /// An unsigned 24-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `u32`.
    ///
    pub const MEDIUMINT_UNSIGNED: Self = Self(9, UNSIGNED);

    /// A 32-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i32`.
    ///
    pub const INT: Self = Self(3, 0);

    /// An unsigned 32-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `u32`.
    ///
    pub const INT_UNSIGNED: Self = Self(3, UNSIGNED);

    /// A 64-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i64`.
    ///
    pub const BIGINT: Self = Self(8, 0);

    /// An unsigned 64-bit integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `u64`.
    ///
    pub const BIGINT_UNSIGNED: Self = Self(8, UNSIGNED);

    /// A 4-byte approximate numeric data value.
    ///
    /// Compatible with `f32` or `f64`.
    ///
    /// Maps to `f32`.
    ///
    pub const FLOAT: Self = Self(4, 0);

    /// An 8-byte approximate numeric data value.
    ///
    /// Compatible with `f32` or `f64`.
    ///
    /// Maps to `f64`.
    ///
    pub const DOUBLE: Self = Self(5, 0);

    /// Always `NULL`.
    ///
    /// Compatible with [`Null`].
    ///
    /// Does not map to any type.
    ///
    /// A `NULL` type is only returned by MySQL when looking at the type
    /// of a parameter.
    ///
    /// A `NULL` type is only sent to MySQL when sending a [`Null`] value. [`Null`]
    /// is a dynamic or type-erased `NULL`. Unlike `Option<_>::None`, [`Null`] can be
    /// used to send a SQL `NULL` without knowing the SQL type.
    ///
    pub const NULL: Self = Self(6, 0);

    /// A fixed-length string that is always right-padded with spaces
    /// to the specified length when stored.
    ///
    pub const CHAR: Self = Self(254, 0);

    /// A fixed-length binary string that is always right-padded with zeroes
    /// to the specified length when stored.
    ///
    /// The type identifier for `BINARY` is the same as `CHAR`. At the type
    /// level, they are identical. At the column, the presence of a binary (`_bin`)
    /// collation determines if the type stores binary data.
    ///
    pub const BINARY: Self = Self(254, 0);

    /// A variable-length string.
    pub const VARCHAR: Self = Self(253, 0);

    /// A variable-length binary string.
    ///
    /// The type identifier for `VARBINARY` is the same as `VARCHAR`. At the type
    /// level, they are identical. At the column, the presence of a binary (`_bin`)
    /// collation determines if the type stores binary data.
    ///
    pub const VARBINARY: Self = Self(253, 0);

    /// A variable-length string that is assumed to be stored by reference.
    pub const TEXT: Self = Self(252, 0);

    /// A variable-length string that is assumed to be stored by reference.
    ///
    /// The type identifier for `BLOB` is the same as `TEXT`. At the type
    /// level, they are identical. At the column, the presence of a binary (`_bin`)
    /// collation determines if the type stores binary data.
    ///
    pub const BLOB: Self = Self(252, 0);
}
