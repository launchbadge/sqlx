use sqlx_core::IsolationLevel;

#[derive(Debug)]
pub struct MySqlTransactionOptions {
    with_consistent_snapshot: bool,
    read_only: bool,
    isolation_level: IsolationLevel,
}

impl MySqlTransactionOptions {
    pub fn read_only(&mut self) -> &mut Self {
        self.read_only = true;
        self
    }

    pub fn with_consistent_snapshot(&mut self) -> &mut Self {
        self.with_consistent_snapshot = true;
        self
    }

    pub fn isolation(&mut self, level: IsolationLevel) -> &mut Self {
        self.isolation_level = level;
        self
    }
}

// impl MySqlTransactionOptions {
//     pub fn begin(&self) -> MySqlTransaction {
//         // [..]
//     }
// }
