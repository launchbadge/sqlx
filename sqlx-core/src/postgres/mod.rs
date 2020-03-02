//! **Postgres** database and connection types.

pub use arguments::PgArguments;
pub use connection::PgConnection;
pub use cursor::PgCursor;
pub use database::Postgres;
pub use error::PgError;
pub use row::{PgRow, PgValue};
pub use types::PgTypeInfo;

mod arguments;
mod connection;
mod cursor;
mod database;
mod error;
mod executor;
mod protocol;
mod row;
mod sasl;
mod stream;
// mod tls;
mod types;

/// An alias for [`Pool`][crate::Pool], specialized for **Postgres**.
pub type PgPool = super::Pool<PgConnection>;

make_query_as!(PgQueryAs, Postgres, PgRow);
impl_map_row_for_row!(Postgres, PgRow);
impl_column_index_for_row!(Postgres);
impl_from_row_for_tuples!(Postgres, PgRow);
