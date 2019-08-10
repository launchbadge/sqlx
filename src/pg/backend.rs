use crate::backend::Backend;

pub struct Pg;

impl Backend for Pg {
    type Connection = super::PgConnection;
    type Row = super::PgRow;
}

// Generates tuple FromRow impls for this backend
impl_from_row_tuples_for_backend!(Pg);
