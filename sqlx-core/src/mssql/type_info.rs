use std::fmt::{self, Display, Formatter};

use crate::mssql::protocol::type_info::TypeInfo as ProtocolTypeInfo;
use crate::type_info::TypeInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsSqlTypeInfo(pub(crate) ProtocolTypeInfo);

impl TypeInfo for MsSqlTypeInfo {}

impl Display for MsSqlTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();
        self.0.fmt(&mut buf);

        f.pad(&*buf)
    }
}
