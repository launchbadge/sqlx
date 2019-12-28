// Modules
pub use sqlx_core::{arguments, decode, describe, encode, error, pool, row, types};

// Types
pub use sqlx_core::{Connection, Database, Error, Executor, FromRow, Pool, Query, Result, Row};

// Functions
pub use sqlx_core::query;

#[cfg(feature = "mysql")]
pub use sqlx_core::mysql::{self, MySql};

#[cfg(feature = "postgres")]
pub use sqlx_core::postgres::{self, Postgres};

#[cfg(feature = "macros")]
#[doc(hidden)]
pub use sqlx_core::{TyCons, TyConsExt};

#[cfg(feature = "macros")]
#[proc_macro_hack::proc_macro_hack(fake_call_site)]
pub use sqlx_macros::query;
