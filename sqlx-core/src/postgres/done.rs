use crate::done::Done;
use crate::postgres::Postgres;
use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct PgDone {
    pub(super) rows_affected: u64,
}

impl Done for PgDone {
    type Database = Postgres;

    fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<PgDone> for PgDone {
    fn extend<T: IntoIterator<Item = PgDone>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}
