use crate::database::Database;
use std::iter::Extend;

pub trait Done: 'static + Sized + Send + Sync + Default + Extend<Self> {
    type Database: Database;

    /// Returns the number of rows affected by an `UPDATE`, `INSERT`, or `DELETE`.
    fn rows_affected(&self) -> u64;
}
