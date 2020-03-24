//! **Postgres** database and connection types.

pub use arguments::PgArguments;
pub use connection::PgConnection;
pub use cursor::PgCursor;
pub use database::Postgres;
pub use error::PgError;
pub use listen::{PgListener, PgNotification};
pub use row::{PgRow, PgValue};
pub use types::PgTypeInfo;

mod arguments;
mod connection;
mod cursor;
mod database;
mod error;
mod executor;
mod listen;
mod protocol;
mod row;
mod sasl;
mod stream;
mod tls;
pub mod types;

/// An alias for [`Pool`][crate::Pool], specialized for **Postgres**.
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub type PgPool = crate::pool::Pool<PgConnection>;

make_query_as!(PgQueryAs, Postgres, PgRow);
impl_map_row_for_row!(Postgres, PgRow);
impl_column_index_for_row!(PgRow);
impl_from_row_for_tuples!(Postgres, PgRow);
