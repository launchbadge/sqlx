use crate::done::Done;
use crate::mysql::MySql;
use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct MySqlDone {
    pub(super) rows_affected: u64,
    pub(super) last_insert_id: u64,
}

impl MySqlDone {
    pub fn last_insert_id(&self) -> u64 {
        self.last_insert_id
    }
}

impl Done for MySqlDone {
    type Database = MySql;

    fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<MySqlDone> for MySqlDone {
    fn extend<T: IntoIterator<Item = MySqlDone>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            self.last_insert_id = elem.last_insert_id;
        }
    }
}
