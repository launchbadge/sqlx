use bytestring::ByteString;
use sqlx_core::{Column, Database};

use crate::{PgTypeInfo, Postgres};

// TODO: inherent methods from <Column>

/// Represents a column from a query in Postgres.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct PgColumn {
    index: usize,
    name: ByteString,
    type_info: PgTypeInfo,
}

impl Column for PgColumn {
    type Database = Postgres;

    fn name(&self) -> &str {
        &self.name
    }

    fn index(&self) -> usize {
        self.index
    }

    fn type_info(&self) -> &PgTypeInfo {
        &self.type_info
    }
}
