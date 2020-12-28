use sqlx_core::{Database, HasOutput, Runtime};

#[derive(Debug)]
pub struct MySql;

impl<Rt> Database<Rt> for MySql
where
    Rt: Runtime,
{
    type Connection = super::MySqlConnection<Rt>;
}

impl<'x> HasOutput<'x> for MySql {
    type Output = &'x mut Vec<u8>;
}
