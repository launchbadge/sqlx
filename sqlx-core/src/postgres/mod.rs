//! **Postgres** database and connection types.

pub use arguments::PgArguments;
pub use connection::PgConnection;
pub use database::Postgres;
pub use error::PgError;
pub use row::PgRow;
#[doc(hidden)]
pub use types::r#struct::{decode_struct_field, encode_struct_field};
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
pub type PgPool = super::Pool<PgConnection>;
