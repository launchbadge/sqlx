use crate::{PgColumn, PgTypeInfo};
use std::borrow::Cow;
use std::sync::Arc;

pub struct PgStatement<'q> {
    sql: Cow<'q, str>,
    metadata: Arc<StatementMetadata>,
}

pub(crate) struct StatementMetadata {
    pub(crate) parameters: Vec<PgTypeInfo>,
    pub(crate) columns: Vec<PgColumn>,
}
