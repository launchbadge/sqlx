//! Generic database driver with the specific driver selected at runtime.

use crate::executor::Executor;

#[macro_use]
mod decode;

#[macro_use]
mod encode;

#[macro_use]
mod r#type;

mod arguments;
pub(crate) mod column;
mod connection;
mod database;
mod error;
mod kind;
mod options;
mod query_result;
pub(crate) mod row;
mod statement;
mod transaction;
pub(crate) mod type_info;
pub mod types;
pub(crate) mod value;

#[cfg(feature = "migrate")]
mod migrate;

pub use arguments::{AnyArgumentBuffer, AnyArguments};
pub use column::{AnyColumn, AnyColumnIndex};
pub use connection::AnyConnection;
// Used internally in `sqlx-macros`
#[doc(hidden)]
pub use connection::AnyConnectionKind;
pub use database::Any;
pub use decode::AnyDecode;
pub use encode::AnyEncode;
pub use kind::AnyKind;
pub use options::AnyConnectOptions;
pub use query_result::AnyQueryResult;
pub use row::AnyRow;
pub use statement::AnyStatement;
pub use transaction::AnyTransactionManager;
pub use type_info::AnyTypeInfo;
pub use value::{AnyValue, AnyValueRef};

pub type AnyPool = crate::pool::Pool<Any>;

pub type AnyPoolOptions = crate::pool::PoolOptions<Any>;

/// An alias for [`Executor<'_, Database = Any>`][Executor].
pub trait AnyExecutor<'c>: Executor<'c, Database = Any> {}
impl<'c, T: Executor<'c, Database = Any>> AnyExecutor<'c> for T {}

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(AnyArguments<'q>);
impl_executor_for_pool_connection!(Any, AnyConnection, AnyRow);
impl_executor_for_transaction!(Any, AnyRow);
impl_acquire!(Any, AnyConnection);
impl_column_index_for_row!(AnyRow);
impl_column_index_for_statement!(AnyStatement);
impl_into_maybe_pool!(Any, AnyConnection);

// required because some databases have a different handling of NULL
impl<'q, T> crate::encode::Encode<'q, Any> for Option<T>
where
    T: AnyEncode<'q> + 'q,
{
    fn encode_by_ref(&self, buf: &mut AnyArgumentBuffer<'q>) -> crate::encode::IsNull {
        match &mut buf.0 {
            #[cfg(feature = "postgres")]
            arguments::AnyArgumentBufferKind::Postgres(args, _) => args.add(self),

            #[cfg(feature = "mysql")]
            arguments::AnyArgumentBufferKind::MySql(args, _) => args.add(self),

            #[cfg(feature = "mssql")]
            arguments::AnyArgumentBufferKind::Mssql(args, _) => args.add(self),

            #[cfg(feature = "sqlite")]
            arguments::AnyArgumentBufferKind::Sqlite(args) => args.add(self),
        }

        // unused
        crate::encode::IsNull::No
    }
}
