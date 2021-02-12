use sqlx_core::database::{HasOutput, HasRawValue};
use sqlx_core::{Database, Runtime};

use super::{
    MySqlColumn, MySqlConnection, MySqlOutput, MySqlQueryResult, MySqlRawValue, MySqlRow,
    MySqlTypeId, MySqlTypeInfo,
};

#[derive(Debug)]
pub struct MySql;

impl<Rt: Runtime> Database<Rt> for MySql {
    type Connection = MySqlConnection<Rt>;

    type Column = MySqlColumn;

    type Row = MySqlRow;

    type QueryResult = MySqlQueryResult;

    type TypeId = MySqlTypeId;

    type TypeInfo = MySqlTypeInfo;
}

impl<'x> HasOutput<'x> for MySql {
    type Output = MySqlOutput<'x>;
}

impl<'r> HasRawValue<'r> for MySql {
    type RawValue = MySqlRawValue<'r>;
}
