//! **PostgreSQL** database driver.

mod arguments;
mod connection;
mod database;
mod error;
mod io;
mod listener;
mod message;
mod options;
mod row;
mod transaction;
mod type_info;
pub mod types;
mod value;

pub use arguments::{PgArgumentBuffer, PgArguments};
pub use connection::PgConnection;
pub use database::Postgres;
pub use error::{PgDatabaseError, PgErrorPosition};
pub use listener::{PgListener, PgNotification};
pub use message::PgSeverity;
pub use options::{PgConnectOptions, PgSslMode};
pub use row::PgRow;
pub use transaction::PgTransactionManager;
pub use type_info::{PgTypeInfo, PgTypeKind};
pub use value::{PgValue, PgValueFormat, PgValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for Postgres.
pub type PgPool = crate::pool::Pool<PgConnection>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(PgArguments);
impl_executor_for_pool_connection!(Postgres, PgConnection, PgRow);
