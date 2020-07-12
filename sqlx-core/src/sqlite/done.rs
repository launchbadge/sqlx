use crate::done::Done;
use crate::sqlite::Sqlite;
use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct SqliteDone {
    pub(super) changes: u64,
    pub(super) last_insert_rowid: i64,
}

impl SqliteDone {
    pub fn last_insert_rowid(&self) -> i64 {
        self.last_insert_rowid
    }
}

impl Done for SqliteDone {
    type Database = Sqlite;

    fn rows_affected(&self) -> u64 {
        self.changes
    }
}

impl Extend<SqliteDone> for SqliteDone {
    fn extend<T: IntoIterator<Item = SqliteDone>>(&mut self, iter: T) {
        for elem in iter {
            self.changes += elem.changes;
            self.last_insert_rowid = elem.last_insert_rowid;
        }
    }
}
