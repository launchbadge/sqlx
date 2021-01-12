use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct MssqlOutcome {
    pub(super) rows_affected: u64,
}

impl MssqlOutcome {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<MssqlOutcome> for MssqlOutcome {
    fn extend<T: IntoIterator<Item = MssqlOutcome>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}

#[cfg(feature = "any")]
impl From<MssqlOutcome> for crate::any::AnyOutcome {
    fn from(done: MssqlOutcome) -> Self {
        crate::any::AnyOutcome {
            rows_affected: done.rows_affected,
            last_insert_id: None,
        }
    }
}
