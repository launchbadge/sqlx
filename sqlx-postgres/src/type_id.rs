/// A unique identifier for a Postgres data type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    any(feature = "offline", feature = "serde"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[allow(clippy::module_name_repetitions)]
pub enum PgTypeId {
    Oid(u32),
    Name(&'static str),
}

// Data Types
// https://www.postgresql.org/docs/current/datatype.html

impl PgTypeId {
    // Boolean
    // https://www.postgresql.org/docs/current/datatype-boolean.html

    /// The SQL standard `boolean` type.
    ///
    /// Maps to `bool`.
    ///
    pub const BOOLEAN: Self = Self::Oid(16);

    // Integers
    // https://www.postgresql.org/docs/current/datatype-numeric.html#DATATYPE-INT

    /// A 2-byte integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i16`.
    ///
    #[doc(alias = "INT2")]
    #[doc(alias = "SMALLSERIAL")]
    pub const SMALLINT: Self = Self::Oid(21);

    /// A 4-byte integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i32`.
    ///
    #[doc(alias = "INT4")]
    #[doc(alias = "SERIAL")]
    pub const INTEGER: Self = Self::Oid(23);

    /// An 8-byte integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i64`.
    ///
    #[doc(alias = "INT8")]
    #[doc(alias = "BIGSERIAL")]
    pub const BIGINT: Self = Self::Oid(20);

    // Arbitrary Precision Numbers
    // https://www.postgresql.org/docs/current/datatype-numeric.html#DATATYPE-NUMERIC-DECIMAL

    /// An exact numeric type with a user-specified precision.
    ///
    /// Compatible with [`bigdecimal::BigDecimal`], [`rust_decimal::Decimal`], [`num_int::BigInt`], and any
    /// primitive integer type. Truncation or loss-of-precision is considered an error
    /// when decoding into the selected Rust integer type.
    ///
    /// With a scale of `0` (e.g, `NUMERIC(17, 0)`), maps to `num_int::BigInt`; otherwise,
    /// maps to [`bigdecimal::BigDecimal`] or [`rust_decimal::Decimal`] (depending on
    /// enabled crate features).
    ///
    #[doc(alias = "DECIMAL")]
    pub const NUMERIC: Self = Self::Oid(1700);

    // Floating-Point
    // https://www.postgresql.org/docs/current/datatype-numeric.html#DATATYPE-FLOAT

    /// A 4-byte floating-point numeric type.
    ///
    /// Compatible with `f32` or `f64`.
    ///
    /// Maps to `f32`.
    ///
    #[doc(alias = "FLOAT4")]
    pub const REAL: Self = Self::Oid(700);

    /// An 8-byte floating-point numeric type.
    ///
    /// Compatible with `f32` or `f64`.
    ///
    /// Maps to `f64`.
    ///
    #[doc(alias = "FLOAT8")]
    pub const DOUBLE: Self = Self::Oid(701);

    /// The `UNKNOWN` Postgres type. Returned for expressions that do not
    /// have a type (e.g., `SELECT $1` with no parameter type hint
    /// or `SELECT NULL`).
    pub const UNKNOWN: Self = Self::Oid(705);
}

impl PgTypeId {
    #[must_use]
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::BOOLEAN => "BOOLEAN",

            Self::SMALLINT => "SMALLINT",
            Self::INTEGER => "INTEGER",
            Self::BIGINT => "BIGINT",

            Self::NUMERIC => "NUMERIC",

            Self::REAL => "REAL",
            Self::DOUBLE => "DOUBLE",

            _ => "UNKNOWN",
        }
    }

    pub(crate) const fn is_integer(&self) -> bool {
        matches!(*self, Self::SMALLINT | Self::INTEGER | Self::BIGINT)
    }
}
