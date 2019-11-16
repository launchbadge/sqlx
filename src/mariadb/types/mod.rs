use super::protocol::{FieldType, ParameterFlag};
use crate::{
    mariadb::MariaDb,
    types::{HasTypeMetadata, TypeMetadata},
};

pub mod binary;
pub mod boolean;
pub mod character;
pub mod numeric;

#[derive(Debug)]
pub struct MariaDbTypeMetadata {
    pub field_type: FieldType,
    pub param_flag: ParameterFlag,
}

impl HasTypeMetadata for MariaDb {
    type TypeMetadata = MariaDbTypeMetadata;
    type TypeId = u8;

    fn param_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
        Some(match FieldType(*id) {
            FieldType::MYSQL_TYPE_TINY => "i8",
            FieldType::MYSQL_TYPE_SHORT => "i16",
            FieldType::MYSQL_TYPE_LONG => "i32",
            FieldType::MYSQL_TYPE_LONGLONG => "i64",
            FieldType::MYSQL_TYPE_VAR_STRING => "&str",
            FieldType::MYSQL_TYPE_FLOAT => "f32",
            FieldType::MYSQL_TYPE_DOUBLE => "f64",
            FieldType::MYSQL_TYPE_BLOB => "&[u8]",
            _ => return None
        })
    }

    fn return_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
        Some(match FieldType(*id) {
            FieldType::MYSQL_TYPE_TINY => "i8",
            FieldType::MYSQL_TYPE_SHORT => "i16",
            FieldType::MYSQL_TYPE_LONG => "i32",
            FieldType::MYSQL_TYPE_LONGLONG => "i64",
            FieldType::MYSQL_TYPE_VAR_STRING => "String",
            FieldType::MYSQL_TYPE_FLOAT => "f32",
            FieldType::MYSQL_TYPE_DOUBLE => "f64",
            FieldType::MYSQL_TYPE_BLOB => "Vec<u8>",
            _ => return None
        })
    }
}

impl TypeMetadata for MariaDbTypeMetadata {
    type TypeId = u8;

    fn type_id(&self) -> &Self::TypeId {
        &self.field_type.0
    }
}
