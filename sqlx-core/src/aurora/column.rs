use crate::aurora::type_info::AuroraTypeInfo;
use crate::aurora::Aurora;
use crate::column::Column;
use crate::ext::ustr::UStr;

#[derive(Debug, Clone)]
pub struct AuroraColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: AuroraTypeInfo,
}

impl crate::column::private_column::Sealed for AuroraColumn {}

impl Column for AuroraColumn {
    type Database = Aurora;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &AuroraTypeInfo {
        &self.type_info
    }
}
