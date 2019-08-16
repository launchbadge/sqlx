use crate::backend::Backend;

pub struct Pg;

impl Backend for Pg {
    type RawConnection = super::PgRawConnection;
    type Row = super::PgRow;
}

// Generates tuple FromRow impls for this backend
impl_from_row_tuples_for_backend!(Pg);
