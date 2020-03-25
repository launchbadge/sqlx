use crate::cursor::HasCursor;
use crate::database::Database;
use crate::mysql::error::MySqlError;
use crate::row::HasRow;
use crate::value::HasRawValue;

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
    type Database = MySql;

    type RawValue = super::MySqlValue<'c>;
}
