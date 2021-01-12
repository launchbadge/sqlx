use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct AnyOutcome {
    pub(crate) rows_affected: u64,
    pub(crate) last_insert_id: Option<i64>,
}

impl AnyOutcome {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }

    pub fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
    }
}

impl Extend<AnyOutcome> for AnyOutcome {
    fn extend<T: IntoIterator<Item = AnyOutcome>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            self.last_insert_id = elem.last_insert_id;
        }
    }
}
