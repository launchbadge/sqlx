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

/// Macro to reduce boilerplate for defining constants for `PgTypeId`. See usage below for examples.
macro_rules! type_id {
    ($(
        $(#[$meta:meta])*
        $name:ident = $kind:ident ($val:literal) $(, [] = $array_kind:ident ($array_val:literal))?
    );* $(;)?) => {
        impl PgTypeId {
            $(
                $(#[$meta])*
                pub const $name: Self = Self::$kind($val);

                $(
                    paste::paste! {
                        #[doc = "An array of [`" $name "`][Self::" $name "]."]
                        ///
                        /// Maps to either a slice or a vector of the equivalent Rust type.
                        pub const [<$name _ARRAY>]: Self = Self::$array_kind($array_val);
                    }
                )?
            )*
        }

        impl PgTypeId {
            /// Get the name of this type as a string.
            #[must_use]
            pub (crate) const fn name(self) -> &'static str {
                match self {
                    $(
                        Self::$name => stringify!($name),
                        $(
                            // just appends `[]` to the type name
                            Self::$array_kind($array_val) => concat!(stringify!($name), "[]"),
                        )?
                    )*
                    Self::Name(name) => name,
                    _ => "UNKNOWN"
                }
            }

            /// Get the ID of the inner type if the current type is an array.
            #[allow(dead_code)]
            pub (crate) const fn elem_type(self) -> Option<Self> {
                match self {
                    // only generates an arm if `$array_kind` and `$array_val` are provided
                    $($(Self::$array_kind($array_val) => Some(Self::$kind($val)),)?)*
                    _ => None,
                }
            }

            /// Get the type ID for an array of this type, if we know it.
            #[allow(dead_code)]
            pub (crate) const fn array_type(self) -> Option<Self> {
                match self {
                    // only generates an arm if `$array_kind` and `$array_val` are provided
                    $($(Self::$name => Some(Self::$array_kind($array_val)),)?)*
                    _ => None,
                }
            }
        }
    };
}

// Data Types
// https://www.postgresql.org/docs/current/datatype.html
// for OIDs see: https://github.com/postgres/postgres/blob/master/src/include/catalog/pg_type.dat
type_id! {
    // Boolean
    // https://www.postgresql.org/docs/current/datatype-boolean.html

    /// The SQL standard `boolean` type.
    ///
    /// Maps to `bool`.
    ///
    BOOLEAN = Oid(16), [] = Oid(1000); // also defines `BOOLEAN_ARRAY` for the `BOOLEAN[]` type

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
    SMALLINT = Oid(21), [] = Oid(1005);

    /// A 4-byte integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i32`.
    ///
    #[doc(alias = "INT4")]
    #[doc(alias = "SERIAL")]
    INTEGER = Oid(23), [] = Oid(1007);

    /// An 8-byte integer.
    ///
    /// Compatible with any primitive integer type.
    ///
    /// Maps to `i64`
    ///
    #[doc(alias = "INT8")]
    #[doc(alias = "BIGSERIAL")]
    BIGINT = Oid(20), [] = Oid(1016);

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
    NUMERIC = Oid(1700), [] = Oid(1231);

    // Floating-Point
    // https://www.postgresql.org/docs/current/datatype-numeric.html#DATATYPE-FLOAT

    /// A 4-byte floating-point numeric type.
    ///
    /// Compatible with `f32` or `f64`.
    ///
    /// Maps to `f32`.
    ///
    #[doc(alias = "FLOAT4")]
    REAL = Oid(700), [] = Oid(1021);

    /// An 8-byte floating-point numeric type.
    ///
    /// Compatible with `f32` or `f64`.
    ///
    /// Maps to `f64`.
    ///
    #[doc(alias = "FLOAT8")]
    DOUBLE = Oid(701), [] = Oid(1022);

    /// The `UNKNOWN` Postgres type. Returned for expressions that do not
    /// have a type (e.g., `SELECT $1` with no parameter type hint
    /// or `SELECT NULL`).
    UNKNOWN = Oid(705);
}

impl PgTypeId {
    pub(crate) const fn oid(self) -> Option<u32> {
        if let Self::Oid(oid) = self {
            Some(oid)
        } else {
            None
        }
    }

    pub(crate) const fn is_integer(self) -> bool {
        matches!(self, Self::SMALLINT | Self::INTEGER | Self::BIGINT)
    }
}
