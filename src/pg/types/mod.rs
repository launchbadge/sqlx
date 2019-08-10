use super::Pg;
use crate::types::TypeMetadata;

mod boolean;
mod character;
mod numeric;

pub use self::boolean::Bool;

pub struct PgTypeMetadata {
    pub oid: u32,
    pub array_oid: u32,
}

impl TypeMetadata for Pg {
    type TypeMetadata = PgTypeMetadata;
}
