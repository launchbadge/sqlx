use super::MariaDB;
use crate::types::TypeMetadata;
use crate::mariadb::FieldType;
use crate::mariadb::protocol::types::ParamFlag;

mod boolean;

pub enum MariaDbTypeFormat {
    Text = 0,
    Binary = 1,
}

pub struct MariaDbTypeMetadata {
    pub format: MariaDbTypeFormat,
    pub field_type: FieldType,
    pub param_flag: ParamFlag,
}

impl TypeMetadata for MariaDb {
    type TypeMetadata = MariaDbTypeMetadata;
}
