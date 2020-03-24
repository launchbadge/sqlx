//! Conversions between Rust and **MySQL** types.
//!
//! # Types
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | TINYINT(1)                                           |
//! | `i8`                                  | TINYINT                                              |
//! | `i16`                                 | SMALLINT                                             |
//! | `i32`                                 | INT                                                  |
//! | `i64`                                 | BIGINT                                               |
//! | `u8`                                  | TINYINT UNSIGNED                                     |
//! | `u16`                                 | SMALLINT UNSIGNED                                    |
//! | `u32`                                 | INT UNSIGNED                                         |
//! | `u64`                                 | BIGINT UNSIGNED                                      |
//! | `f32`                                 | FLOAT                                                |
//! | `f64`                                 | DOUBLE                                               |
//! | `&str`, `String`                      | VARCHAR, CHAR, TEXT                                  |
//! | `&[u8]`, `Vec<u8>`                    | VARBINARY, BINARY, BLOB                              |
//!
//! ### [`chrono`](https://crates.io/crates/chrono)
//!
//! Requires the `chrono` Cargo feature flag.
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `chrono::DateTime<Utc>`               | TIMESTAMP                                            |
//! | `chrono::DateTime<Local>`             | TIMETAMP                                             |
//! | `chrono::NaiveDateTime`               | DATETIME                                             |
//! | `chrono::NaiveDate`                   | DATE                                                 |
//! | `chrono::NaiveTime`                   | TIME                                                 |
//!
//! ### [`time`](https://crates.io/crates/time)
//!
//! Requires the `time` Cargo feature flag.
//!
//! | Rust type                             | MySQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `time::PrimitiveDateTime`             | DATETIME                                             |
//! | `time::OffsetDateTime`                | TIMESTAMP                                            |
//! | `time::Date`                          | DATE                                                 |
//! | `time::Time`                          | TIME                                                 |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from MySQL.
//!

mod bool;
mod bytes;
mod float;
mod int;
mod str;
mod uint;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "time")]
mod time;

use std::fmt::{self, Debug, Display};

use crate::decode::Decode;
use crate::mysql::protocol::TypeId;
use crate::mysql::protocol::{ColumnDefinition, FieldFlags};
use crate::mysql::{MySql, MySqlValue};
use crate::types::TypeInfo;

#[derive(Clone, Debug, Default)]
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

    pub(crate) fn from_column_def(def: &ColumnDefinition) -> Option<Self> {
        if def.type_id == TypeId::NULL {
            return None;
        }

        Some(Self {
            id: def.type_id,
            is_unsigned: def.flags.contains(FieldFlags::UNSIGNED),
            is_binary: def.flags.contains(FieldFlags::BINARY),
            char_set: def.char_set,
        })
    }

    #[doc(hidden)]
    pub fn type_name(&self) -> &'static str {
        self.id.type_name()
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
        if self.id.type_name() != "<unknown>" {
            write!(f, "{}", self.id.type_name())
        } else {
            write!(f, "ID {:#x}", self.id.0)
        }
    }
}

impl TypeInfo for MySqlTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
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
                true
            }

            // Fallback to equality of only [id] and [is_unsigned]
            _ => self.id.0 == other.id.0 && self.is_unsigned == other.is_unsigned,
        }
    }
}

impl<'de, T> Decode<'de, MySql> for Option<T>
where
    T: Decode<'de, MySql>,
{
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        value
            .map(|value| <T as Decode<MySql>>::decode(Some(value)))
            .transpose()
    }
}
