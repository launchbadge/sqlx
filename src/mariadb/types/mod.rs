use super::MariaDB;
use crate::{
    mariadb::{protocol::types::ParamFlag, FieldType},
    types::TypeMetadata,
};

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
