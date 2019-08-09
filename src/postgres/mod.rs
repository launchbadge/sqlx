use crate::backend::Backend;

mod connection;

pub use connection::RawConnection;

mod protocol;

pub mod types;

pub struct Postgres;

impl Backend for Postgres {
    type RawConnection = RawConnection;
    type RawRow = protocol::DataRow;
    type TypeMetadata = types::TypeMetadata;
}

// Generates tuple FromRow impls for this backend
impl_from_row_tuples_for_backend!(Postgres);
