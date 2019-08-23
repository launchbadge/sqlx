use crate::backend::Backend;

pub struct Postgres;

impl Backend for Postgres {
    type QueryParameters = super::PostgresQueryParameters;
    type RawConnection = super::PostgresRawConnection;
    type Row = super::PostgresRow;
}

// Generates tuple FromSqlRow impls for this backend
impl_from_sql_row_tuples_for_backend!(Postgres);
