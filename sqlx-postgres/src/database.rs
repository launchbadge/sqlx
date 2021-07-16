use sqlx_core::database::{HasOutput, HasRawValue};
use sqlx_core::placeholders;
use sqlx_core::Database;

use super::{PgColumn, PgOutput, PgQueryResult, PgRawValue, PgRow, PgTypeId, PgTypeInfo};

#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Column = PgColumn;

    type Row = PgRow;

    type QueryResult = PgQueryResult;

    type TypeInfo = PgTypeInfo;

    type TypeId = PgTypeId;

    const PLACEHOLDER_CHAR: char = '$';
    const PARAM_INDEXING: placeholders::ParamIndexing = placeholders::ParamIndexing::OneIndexed;
}

// 'x: execution
impl<'x> HasOutput<'x> for Postgres {
    type Output = PgOutput<'x>;
}

// 'r: row
impl<'r> HasRawValue<'r> for Postgres {
    type Database = Self;
    type RawValue = PgRawValue<'r>;
}
