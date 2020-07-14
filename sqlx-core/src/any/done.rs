use crate::any::Any;
use crate::done::Done;
use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct AnyDone {
    pub(crate) rows_affected: u64,
    pub(crate) last_insert_id: Option<i64>,
}

impl AnyDone {
    pub fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
    }
}

impl Done for AnyDone {
    type Database = Any;

    fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<AnyDone> for AnyDone {
    fn extend<T: IntoIterator<Item = AnyDone>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            self.last_insert_id = elem.last_insert_id;
        }
    }
}
