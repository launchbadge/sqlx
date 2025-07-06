use super::MySqlColumn;
use crate::column::ColumnIndex;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::HashMap;
use crate::{MySql, MySqlArguments, MySqlTypeInfo};
use either::Either;
use sqlx_core::sql_str::SqlStr;
use std::sync::Arc;

pub(crate) use sqlx_core::statement::*;

#[derive(Debug, Clone)]
pub struct MySqlStatement {
    pub(crate) sql: SqlStr,
    pub(crate) metadata: MySqlStatementMetadata,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MySqlStatementMetadata {
    pub(crate) columns: Arc<Vec<MySqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
    pub(crate) parameters: usize,
}

impl Statement for MySqlStatement {
    type Database = MySql;

    fn into_sql(self) -> SqlStr {
        self.sql
    }

    fn sql(&self) -> &SqlStr {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[MySqlTypeInfo], usize>> {
        Some(Either::Right(self.metadata.parameters))
    }

    fn columns(&self) -> &[MySqlColumn] {
        &self.metadata.columns
    }

    impl_statement_query!(MySqlArguments);
}

impl ColumnIndex<MySqlStatement> for &'_ str {
    fn index(&self, statement: &MySqlStatement) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .copied()
    }
}
