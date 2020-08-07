use crate::PgTypeInfo;

#[derive(Debug)]
pub struct PgColumn {
    pub(crate) name: String,
    pub(crate) type_info: PgTypeInfo,
}
