use crate::backend::Backend;

pub struct Pg;

impl Backend for Pg {}

// Generates tuple FromRow impls for this backend
impl_from_row_tuples_for_backend!(Pg);
