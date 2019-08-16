use crate::{connection::RawConnection, row::Row};

/// A database backend.
///
/// This trait is used to both allow distinct implementations of traits (
/// e.g., implementing `ToSql for Uuid` differently for MySQL and Postgres) and
/// to query capabilities within a database backend (e.g., with a specific
/// `Connection` can we `bind` a `i64`?).
pub trait Backend: Sized {
    type RawConnection: RawConnection<Backend = Self>;
    type Row: Row<Backend = Self>;
}
