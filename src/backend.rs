use crate::row::RawRow;

pub trait Backend {
    type RawRow: RawRow;

    /// The type used to represent metadata associated with a SQL type.
    type TypeMetadata;
}
