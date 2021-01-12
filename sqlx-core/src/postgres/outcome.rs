use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct PgOutcome {
    pub(super) rows_affected: u64,
}

impl PgOutcome {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<PgOutcome> for PgOutcome {
    fn extend<T: IntoIterator<Item = PgOutcome>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}

#[cfg(feature = "any")]
impl From<PgOutcome> for crate::any::AnyOutcome {
    fn from(done: PgOutcome) -> Self {
        crate::any::AnyOutcome {
            rows_affected: done.rows_affected,
            last_insert_id: None,
        }
    }
}
