use crate::{connection::RawConnection, query::QueryParameters, row::Row};

/// A database backend.
///
/// This trait represents the concept of a backend (e.g. "MySQL" vs "SQLite").
pub trait Backend: Sized {
    /// The concrete `QueryParameters` implementation for this backend.
    type QueryParameters: QueryParameters<Backend = Self>;

    /// The concrete `RawConnection` implementation for this backend.
    type RawConnection: RawConnection<Backend = Self>;

    /// The concrete `Row` implementation for this backend. This type is returned
    /// from methods in the `RawConnection`.
    type Row: Row<Backend = Self>;
}
