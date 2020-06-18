use std::fmt::{self, Display, Formatter};

use crate::mssql::protocol::type_info::TypeInfo as ProtocolTypeInfo;
use crate::type_info::TypeInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MssqlTypeInfo(pub(crate) ProtocolTypeInfo);

impl TypeInfo for MssqlTypeInfo {
    fn name(&self) -> &str {
        self.0.name()
    }
}

impl Display for MssqlTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}
