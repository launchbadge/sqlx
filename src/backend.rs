pub(crate) use self::internal::BackendAssocRawQuery;
use crate::{connection::RawConnection, query::RawQuery, row::Row};

mod internal {
    pub trait BackendAssocRawQuery<'q, DB>
    where
        DB: super::Backend,
    {
        type RawQuery: super::RawQuery<'q, Backend = DB>;
    }
}

/// A database backend.
///
/// This trait is used to both allow distinct implementations of traits (
/// e.g., implementing `ToSql for Uuid` differently for MySQL and Postgres) and
/// to query capabilities within a database backend (e.g., with a specific
/// `Connection` can we `bind` a `i64`?).
pub trait Backend: Sized + for<'q> BackendAssocRawQuery<'q, Self> {
    type RawConnection: RawConnection<Backend = Self>;
    type Row: Row<Backend = Self>;
}
