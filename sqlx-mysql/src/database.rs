use sqlx_core::{Database, HasOutput, Runtime};

use super::{MySqlConnection, MySqlRow, MySqlColumn, MySqlQueryResult};

#[derive(Debug)]
pub struct MySql;

impl<Rt: Runtime> Database<Rt> for MySql {
    type Connection = MySqlConnection<Rt>;

    type Row = MySqlRow;

    type Column = MySqlColumn;

    type QueryResult = MySqlQueryResult;
}

impl<'x> HasOutput<'x> for MySql {
    type Output = &'x mut Vec<u8>;
}
