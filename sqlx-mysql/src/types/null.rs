use sqlx_core::{Null, Type};

use crate::{MySql, MySqlTypeId};

impl Type<MySql> for Null {
    fn type_id() -> MySqlTypeId
    where
        Self: Sized,
    {
        MySqlTypeId::NULL
    }
}
