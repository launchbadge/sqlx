use crate::Database;

/// **MySQL** database driver.
pub struct MySql;

impl Database for MySql {
    type Connection = super::MySqlConnection;

    type Arguments = super::MySqlArguments;

    type Row = super::MySqlRow;
}

impl_into_arguments_for_database!(MySql);
