use std::fmt::{self, Display};

use crate::types::TypeInfo;

// mod bool;
// mod bytes;
// mod float;
mod int;
// mod str;

#[derive(Debug, Clone)]
pub struct SqliteTypeInfo {}

impl Display for SqliteTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl TypeInfo for SqliteTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        todo!()
    }
}
