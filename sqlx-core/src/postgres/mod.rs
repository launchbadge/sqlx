//! **Postgres** database and connection types.

pub use arguments::PgArguments;
pub use buffer::PgRawBuffer;
pub use connection::PgConnection;
pub use cursor::PgCursor;
pub use database::Postgres;
pub use error::PgError;
pub use listen::{PgListener, PgNotification};
pub use row::PgRow;
pub use type_info::PgTypeInfo;
pub use value::{PgData, PgValue};

mod arguments;
mod buffer;
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
mod type_info;
pub mod types;
mod value;

/// An alias for [`Pool`][crate::pool::Pool], specialized for **Postgres**.
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub type PgPool = crate::pool::Pool<PgConnection>;

make_query_as!(PgQueryAs, Postgres, PgRow);
impl_map_row_for_row!(Postgres, PgRow);
impl_from_row_for_tuples!(Postgres, PgRow);
