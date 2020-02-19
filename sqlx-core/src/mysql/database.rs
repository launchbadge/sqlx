use crate::database::{Database, HasCursor, HasRawValue, HasRow};

/// **MySQL** database driver.
pub struct MySql;

impl Database for MySql {
    type Connection = super::MySqlConnection;

    type Arguments = super::MySqlArguments;

    type TypeInfo = super::MySqlTypeInfo;

    type TableId = Box<str>;
}

impl HasRow for MySql {
    type Database = MySql;

    type Row = super::MySqlRow;
}

impl<'a> HasCursor<'a> for MySql {
    type Database = MySql;

    type Cursor = super::MySqlCursor<'a>;
}

impl<'a> HasRawValue<'a> for MySql {
    type RawValue = Option<&'a [u8]>;
}
