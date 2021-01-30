use std::fmt::Debug;

/// Represents the execution result of an operation on the database server.
///
/// Returned from [`execute()`][crate::Executor::execute].
///
pub trait QueryResult: 'static + Sized + Debug + Extend<Self> {
    /// Returns the number of rows changed, deleted, or inserted by the statement
    /// if it was an `UPDATE`, `DELETE` or `INSERT`. For `SELECT` statements, returns
    /// the number of rows returned.
    fn rows_affected(&self) -> u64;
}
