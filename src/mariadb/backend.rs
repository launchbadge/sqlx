use crate::backend::Backend;

#[derive(Debug)]
pub struct MariaDb;

impl Backend for MariaDb {
    type QueryParameters = super::MariaDbQueryParameters;
    type RawConnection = super::MariaDbRawConnection;
    type Row = super::MariaDbRow;
}

impl_from_sql_row_tuples_for_backend!(MariaDb);
impl_into_query_parameters_for_backend!(MariaDb);
