use std::fmt::{self, Display, Formatter};

pub(crate) use sqlx_core::type_info::*;

/// Type information for a MSSQL type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MssqlTypeInfo {
    pub(crate) name: String,
}

impl MssqlTypeInfo {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Return the base type name without any parenthesized precision/scale.
    ///
    /// e.g. `"DECIMAL(10,2)"` → `"DECIMAL"`, `"NVARCHAR(4000)"` → `"NVARCHAR"`
    pub(crate) fn base_name(&self) -> &str {
        self.name.split('(').next().unwrap_or(&self.name).trim()
    }
}

impl Display for MssqlTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(&self.name)
    }
}

impl TypeInfo for MssqlTypeInfo {
    fn is_null(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Map a tiberius column type to a MSSQL type name string.
pub(crate) fn type_name_for_tiberius(col_type: &tiberius::ColumnType) -> &'static str {
    match col_type {
        tiberius::ColumnType::Null => "NULL",
        tiberius::ColumnType::Bit => "BIT",
        tiberius::ColumnType::Int1 => "TINYINT",
        tiberius::ColumnType::Int2 => "SMALLINT",
        tiberius::ColumnType::Int4 => "INT",
        tiberius::ColumnType::Int8 => "BIGINT",
        tiberius::ColumnType::Float4 => "REAL",
        tiberius::ColumnType::Float8 => "FLOAT",
        tiberius::ColumnType::Datetime | tiberius::ColumnType::Datetimen => "DATETIME",
        tiberius::ColumnType::Datetime2 => "DATETIME2",
        tiberius::ColumnType::Datetime4 => "SMALLDATETIME",
        tiberius::ColumnType::DatetimeOffsetn => "DATETIMEOFFSET",
        tiberius::ColumnType::Daten => "DATE",
        tiberius::ColumnType::Timen => "TIME",
        tiberius::ColumnType::Decimaln | tiberius::ColumnType::Numericn => "DECIMAL",
        tiberius::ColumnType::Money => "MONEY",
        tiberius::ColumnType::Money4 => "SMALLMONEY",
        tiberius::ColumnType::BigVarChar | tiberius::ColumnType::NVarchar => "NVARCHAR",
        tiberius::ColumnType::BigChar | tiberius::ColumnType::NChar => "NCHAR",
        tiberius::ColumnType::BigVarBin => "VARBINARY",
        tiberius::ColumnType::BigBinary => "BINARY",
        tiberius::ColumnType::Text | tiberius::ColumnType::NText => "NTEXT",
        tiberius::ColumnType::Image => "IMAGE",
        tiberius::ColumnType::Xml => "XML",
        tiberius::ColumnType::Guid => "UNIQUEIDENTIFIER",
        tiberius::ColumnType::Intn => "INT",
        tiberius::ColumnType::Bitn => "BIT",
        tiberius::ColumnType::Floatn => "FLOAT",
        tiberius::ColumnType::SSVariant => "SQL_VARIANT",
        _ => "UNKNOWN",
    }
}
