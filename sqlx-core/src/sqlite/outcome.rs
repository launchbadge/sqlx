use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct SqliteOutcome {
    pub(super) changes: u64,
    pub(super) last_insert_rowid: i64,
}

impl SqliteOutcome {
    pub fn rows_affected(&self) -> u64 {
        self.changes
    }

    pub fn last_insert_rowid(&self) -> i64 {
        self.last_insert_rowid
    }
}

impl Extend<SqliteOutcome> for SqliteOutcome {
    fn extend<T: IntoIterator<Item = SqliteOutcome>>(&mut self, iter: T) {
        for elem in iter {
            self.changes += elem.changes;
            self.last_insert_rowid = elem.last_insert_rowid;
        }
    }
}

#[cfg(feature = "any")]
impl From<SqliteOutcome> for crate::any::AnyOutcome {
    fn from(done: SqliteOutcome) -> Self {
        crate::any::AnyOutcome {
            rows_affected: done.changes,
            last_insert_id: Some(done.last_insert_rowid),
        }
    }
}
