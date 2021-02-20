/// Transaction isolation level; controls the degree of locking that occurs
/// when selecting data.
///
/// See <https://en.wikipedia.org/wiki/Isolation_(database_systems)#Isolation_levels>.
///
pub enum IsolationLevel {
    /// The lowest isolation level. Dirty reads are allowed, so one transaction
    /// may see **not yet committed** changes made by other transactions.
    ReadUncommitted,

    /// A `SELECT` query will only see data that has been committed before the
    /// query began.
    ///
    /// However, two successive `SELECT` queries can see different data,
    /// even though they are within a single transaction, if a concurrent
    /// transaction has committed in-between.
    ReadCommitted,

    /// A `SELECT` query will only see data committed before the transaction
    /// began.
    RepeatableRead,

    Serializable,
}
