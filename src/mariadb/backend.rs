use crate::backend::{Backend, BackendAssocRawQuery};

pub struct MariaDB;

impl<'q> BackendAssocRawQuery<'q, MariaDB> for MariaDB {
    type RawQuery = super::MariaDbRawQuery<'q>;
}

impl Backend for MariaDB {
    type RawConnection = super::MariaDbRawConnection;
    type Row = super::MariaDbRow;
}

impl_from_sql_row_tuples_for_backend!(MariaDb);
