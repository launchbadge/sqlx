use crate::MySqlTypeId;

/// Provides information about a MySQL type.
#[derive(Debug, Clone)]
#[cfg_attr(
    any(feature = "offline", feature = "serde"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MySqlTypeInfo {
    id: MySqlTypeId,
    flags: u16,
    charset: u8,

    // [max_size] for integer types, this is (M) in BIT(M) or TINYINT(M)
    max_size: u8,
}

impl MySqlTypeInfo {
    /// Returns the unique identifier for this MySQL type.
    pub const fn id(&self) -> MySqlTypeId {
        self.id
    }

    /// Returns `true` if this is the `NULL` type.
    ///
    /// For MySQL, this occurs in types from parameters or when `NULL` is
    /// directly used in an expression by itself, such as `SELECT NULL`.
    ///
    pub const fn is_null(&self) -> bool {
        self.id().is_null()
    }

    /// Returns the name for this MySQL data type.
    pub const fn name(&self) -> &'static str {
        self.id().name()
    }
}
o
