//! **Postgres** database and connection types.

pub use arguments::PgArguments;
pub use connection::PgConnection;
pub use database::Postgres;
pub use error::PgError;
pub use row::PgRow;
pub use types::PgTypeInfo;

mod arguments;
mod connection;
mod database;
mod error;
mod executor;
mod protocol;
mod row;
mod types;

/// An alias for [`Pool`], specialized for **Postgres**.
pub type PgPool = super::Pool<Postgres>;

use std::convert::TryInto;

use crate::url::Url;

// used in tests and hidden code in examples
#[doc(hidden)]
pub async fn connect<T>(url: T) -> crate::Result<PgConnection>
where
    T: TryInto<Url, Error = crate::Error>,
{
    PgConnection::open(url.try_into()).await
}
