use crate::database::Database;

/// **Postgres** database driver.
pub struct Postgres;

impl Database for Postgres {
    type Connection = super::PgConnection;

    type Arguments = super::PgArguments;

    type Row = super::PgRow;
}

impl_into_arguments_for_database!(Postgres);
