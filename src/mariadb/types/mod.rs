use crate::types::TypeMetadata;
use super::protocol::FieldType;
use super::protocol::ParamFlag;
use super::backend::MariaDb;

mod boolean;

pub struct MariaDbTypeMetadata {
    pub field_type: FieldType,
    pub param_flag: ParamFlag,
}

impl TypeMetadata for MariaDb {
    type TypeMetadata = MariaDbTypeMetadata;
}
