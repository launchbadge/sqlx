use sqlx_core::database::{HasOutput, HasRawValue};
use sqlx_core::Database;

use super::{
    MySqlColumn, MySqlOutput, MySqlQueryResult, MySqlRawValue, MySqlRow, MySqlTypeId, MySqlTypeInfo,
};

#[derive(Debug)]
pub struct MySql;

impl Database for MySql {
    type Column = MySqlColumn;

    type Row = MySqlRow;

    type QueryResult = MySqlQueryResult;

    type TypeInfo = MySqlTypeInfo;

    type TypeId = MySqlTypeId;
}

impl<'x> HasOutput<'x> for MySql {
    type Output = MySqlOutput<'x>;
}

impl<'r> HasRawValue<'r> for MySql {
    type RawValue = MySqlRawValue<'r>;
}
