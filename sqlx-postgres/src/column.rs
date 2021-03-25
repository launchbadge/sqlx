use std::num::{NonZeroI16, NonZeroI32};

use bytestring::ByteString;
use sqlx_core::{Column, Database};

use crate::protocol::backend::Field;
use crate::{PgRawValueFormat, PgTypeId, PgTypeInfo, Postgres};

/// Represents a column from a query in Postgres.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct PgColumn {
    /// The index of the column in the row where it is from.
    index: usize,

    /// The name of the column.
    name: ByteString,

    /// The type information for the column's data type.
    pub(crate) type_info: PgTypeInfo,

    /// If the column can be identified as a column of a specific table, the object ID of the table.
    #[cfg_attr(feature = "offline", serde(skip))]
    pub(crate) relation_id: Option<NonZeroI32>,

    /// If the column can be identified as a column of a specific table, the attribute number of the column.
    #[cfg_attr(feature = "offline", serde(skip))]
    pub(crate) relation_attribute_no: Option<NonZeroI16>,

    /// The type size (see pg_type.typlen). Note that negative values denote variable-width types.
    pub(crate) type_size: i16,

    /// The type modifier (see pg_attribute.atttypmod). The meaning of the modifier is type-specific.
    pub(crate) type_modifier: i32,

    /// The format code being used for the column.
    pub(crate) format: PgRawValueFormat,
}

impl PgColumn {
    pub(crate) fn from_field(index: usize, field: Field) -> Self {
        Self {
            index,
            name: field.name,
            type_info: PgTypeInfo(PgTypeId::Oid(field.type_id)),
            relation_id: field.relation_id,
            relation_attribute_no: field.relation_attribute_no,
            type_modifier: field.type_modifier,
            type_size: field.type_size,
            format: field.format,
        }
    }

    /// Returns the name of the column.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the (zero-based) position of the column.
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns type information of the column.
    pub fn type_info(&self) -> &PgTypeInfo {
        &self.type_info
    }
}

impl Column for PgColumn {
    type Database = Postgres;

    #[inline]
    fn name(&self) -> &str {
        self.name()
    }

    #[inline]
    fn index(&self) -> usize {
        self.index()
    }

    #[inline]
    fn type_info(&self) -> &PgTypeInfo {
        self.type_info()
    }
}
