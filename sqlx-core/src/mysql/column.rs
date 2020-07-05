use crate::column::Column;
use crate::ext::ustr::UStr;
use crate::mysql::{MySql, MySqlTypeInfo};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MySqlColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: MySqlTypeInfo,
}

impl crate::column::private_column::Sealed for MySqlColumn {}

impl Column for MySqlColumn {
    type Database = MySql;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &MySqlTypeInfo {
        &self.type_info
    }
}
