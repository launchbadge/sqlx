use crate::done::Done;
use crate::mssql::Mssql;
use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct MssqlDone {
    pub(super) rows_affected: u64,
}

impl Done for MssqlDone {
    type Database = Mssql;

    fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<MssqlDone> for MssqlDone {
    fn extend<T: IntoIterator<Item = MssqlDone>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}
