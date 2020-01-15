use crate::Database;

/// **MySQL** database driver.
pub struct MySql;

impl Database for MySql {
    type Connection = super::MySqlConnection;

    type Arguments = super::MySqlArguments;

    type Row = super::MySqlRow;

    type TypeInfo = super::MySqlTypeInfo;

    type TableId = Box<str>;
}
