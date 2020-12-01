use crate::aurora::Aurora;
use crate::done::Done;
use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct AuroraDone {
    pub(super) rows_affected: u64,
}

impl Done for AuroraDone {
    type Database = Aurora;

    fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<AuroraDone> for AuroraDone {
    fn extend<T: IntoIterator<Item = AuroraDone>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}
