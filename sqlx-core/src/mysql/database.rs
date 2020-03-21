use crate::database::{Database, HasCursor, HasRawValue, HasRow};
use crate::mysql::error::MySqlError;

/// **MySQL** database driver.
#[derive(Debug)]
pub struct MySql;

impl Database for MySql {
    type Connection = super::MySqlConnection;

    type Arguments = super::MySqlArguments;

    type TypeInfo = super::MySqlTypeInfo;

    type TableId = Box<str>;

    type RawBuffer = Vec<u8>;

    type Error = MySqlError;
}

impl<'c> HasRow<'c> for MySql {
    type Database = MySql;

    type Row = super::MySqlRow<'c>;
}

impl<'c, 'q> HasCursor<'c, 'q> for MySql {
    type Database = MySql;

    type Cursor = super::MySqlCursor<'c, 'q>;
}

impl<'c> HasRawValue<'c> for MySql {
    type RawValue = Option<super::MySqlValue<'c>>;
}
