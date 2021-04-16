//! **PostgreSQL** database driver.

mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod io;

#[cfg(not(feature = "_rt-wasm-bindgen"))]
mod listener;
#[cfg(feature = "_rt-wasm-bindgen")]
mod ws_listener;

mod message;
mod options;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

#[cfg(all(feature = "migrate", not(feature = "_rt-wasm-bindgen")))]
mod migrate;

pub use arguments::{PgArgumentBuffer, PgArguments};
pub use column::PgColumn;
pub use connection::PgConnection;
pub use database::Postgres;
pub use error::{PgDatabaseError, PgErrorPosition};

#[cfg(not(feature = "_rt-wasm-bindgen"))]
pub use listener::{PgListener, PgNotification};
#[cfg(feature = "_rt-wasm-bindgen")]
pub use ws_listener::PgListener;

pub use message::PgSeverity;
pub use options::{PgConnectOptions, PgSslMode};
pub use query_result::PgQueryResult;
pub use row::PgRow;
pub use statement::PgStatement;
pub use transaction::PgTransactionManager;
pub use type_info::{PgTypeInfo, PgTypeKind};
pub use value::{PgValue, PgValueFormat, PgValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for Postgres.
#[cfg(not(feature = "_rt-wasm-bindgen"))]
pub type PgPool = crate::pool::Pool<Postgres>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for Postgres.
#[cfg(not(feature = "_rt-wasm-bindgen"))]
pub type PgPoolOptions = crate::pool::PoolOptions<Postgres>;

impl_into_arguments_for_arguments!(PgArguments);

#[cfg(not(feature = "_rt-wasm-bindgen"))]
impl_executor_for_pool_connection!(Postgres, PgConnection, PgRow);

impl_executor_for_transaction!(Postgres, PgRow);

#[cfg(not(feature = "_rt-wasm-bindgen"))]
impl_acquire!(Postgres, PgConnection);

impl_column_index_for_row!(PgRow);
impl_column_index_for_statement!(PgStatement);
impl_into_maybe_pool!(Postgres, PgConnection);
impl_encode_for_option!(Postgres);
