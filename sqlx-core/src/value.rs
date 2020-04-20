use crate::database::Database;

pub trait RawValue<'r> {
    type Database: Database;

    // TODO: fn type_info(&self) -> Option<<Self::Database as Database>::TypeInfo>;
}
