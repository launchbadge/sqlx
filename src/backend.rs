use crate::{connection::RawConnection, row::RawRow};

pub trait Backend {
    type RawConnection: RawConnection;

    type RawRow: RawRow;

    /// The type used to represent metadata associated with a SQL type.
    type TypeMetadata;
}
