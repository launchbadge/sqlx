use sqlx_core::HasOutput;
use sqlx_mysql::{MySqlColumn, MySqlQueryResult, MySqlRow};

use super::MySqlConnection;
use crate::{Database, Runtime};

#[derive(Debug)]
pub struct MySql;

impl<Rt: Runtime> Database<Rt> for MySql {
    type Connection = MySqlConnection<Rt>;
    type Column = MySqlColumn;
    type Row = MySqlRow;
    type QueryResult = MySqlQueryResult;
}

impl<'x> HasOutput<'x> for MySql {
    type Output = &'x mut Vec<u8>;
}
