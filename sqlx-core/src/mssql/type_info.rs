use std::fmt::{self, Display, Formatter};

use crate::type_info::TypeInfo;

#[derive(Debug, Clone)]
pub struct MsSqlTypeInfo {}

impl TypeInfo for MsSqlTypeInfo {}

impl Display for MsSqlTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl PartialEq<MsSqlTypeInfo> for MsSqlTypeInfo {
    fn eq(&self, other: &MsSqlTypeInfo) -> bool {
        unimplemented!()
    }
}

impl Eq for MsSqlTypeInfo {}
