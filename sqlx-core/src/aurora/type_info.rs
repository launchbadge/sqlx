use crate::type_info::TypeInfo;

use rusoto_rds_data::Field;
use std::fmt::{self, Display, Formatter};

/// Type information for an Aurora type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuroraTypeInfo(pub(crate) AuroraType);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuroraType {
    Blob,
    Boolean,
    BooleanArray,
    Double,
    DoubleArray,
    Long,
    LongArray,
    String,
    StringArray,
    Decimal,
    DecimalArray,
    Timestamp,
    TimestampArray,
    Date,
    DateArray,
    Time,
    TimeArray,
}

impl AuroraType {
    pub(crate) fn display_name(&self) -> &str {
        match self {
            AuroraType::Blob => "BLOB",
            AuroraType::Boolean => "BOOL",
            AuroraType::BooleanArray => "BOOL[]",
            AuroraType::Double => "DOUBLE",
            AuroraType::DoubleArray => "DOUBLE[]",
            AuroraType::Long => "LONG",
            AuroraType::LongArray => "LONG[]",
            AuroraType::String => "STRING",
            AuroraType::StringArray => "STRING[]",
            AuroraType::Decimal => "DECIMAL",
            AuroraType::DecimalArray => "DECIMAL[]",
            AuroraType::Timestamp => "TIMESTAMP",
            AuroraType::TimestampArray => "TIMESTAMP[]",
            AuroraType::Date => "DATE",
            AuroraType::DateArray => "DATE[]",
            AuroraType::Time => "TIME",
            AuroraType::TimeArray => "TIME[]",
        }
    }
}

impl TypeInfo for AuroraTypeInfo {
    fn name(&self) -> &str {
        self.0.display_name()
    }

    fn is_null(&self) -> bool {
        false
    }
}

impl Display for AuroraTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}

impl From<&Field> for AuroraTypeInfo {
    fn from(field: &Field) -> Self {
        if let Some(array_value) = &field.array_value {
            if array_value.boolean_values.is_some() {
                AuroraTypeInfo(AuroraType::BooleanArray)
            } else if array_value.double_values.is_some() {
                AuroraTypeInfo(AuroraType::DoubleArray)
            } else if array_value.long_values.is_some() {
                AuroraTypeInfo(AuroraType::LongArray)
            } else {
                AuroraTypeInfo(AuroraType::StringArray)
            }
        } else if field.blob_value.is_some() {
            AuroraTypeInfo(AuroraType::Blob)
        } else if field.boolean_value.is_some() {
            AuroraTypeInfo(AuroraType::Boolean)
        } else if field.double_value.is_some() {
            AuroraTypeInfo(AuroraType::Double)
        } else if field.long_value.is_some() {
            AuroraTypeInfo(AuroraType::Long)
        } else {
            AuroraTypeInfo(AuroraType::String)
        }
    }
}
