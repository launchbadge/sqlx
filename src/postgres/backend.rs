use crate::backend::{Backend, BackendAssocRawQuery};

pub struct Postgres;

impl<'q> BackendAssocRawQuery<'q, Postgres> for Postgres {
    type RawQuery = super::PostgresRawQuery<'q>;
}

impl Backend for Postgres {
    type RawConnection = super::PostgresRawConnection;
    type Row = super::PostgresRow;
}

// Generates tuple FromSqlRow impls for this backend
impl_from_sql_row_tuples_for_backend!(Postgres);
