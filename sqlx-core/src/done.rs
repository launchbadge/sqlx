pub struct Done {
    pub(crate) rows_affected: u64,
    pub(crate) last_insert_id: Option<i64>,
}

impl Done {
    /// Returns the number of rows affected by an `UPDATE`, `INSERT`, or `DELETE`.
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }

    // None if not supported by the driver or there wasn't an
    // ID that was inserted.
    pub fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
    }
}
