mod arguments;
mod connection;
mod database;
mod decode;
mod encode;
mod options;
mod row;
mod transaction;
mod type_info;
mod types;
mod value;

pub use arguments::{AnyArgumentBuffer, AnyArguments};
pub use connection::AnyConnection;
pub use database::Any;
pub use decode::AnyDecode;
pub use encode::AnyEncode;
pub use options::AnyConnectOptions;
pub use row::AnyRow;
pub use transaction::AnyTransactionManager;
pub use type_info::AnyTypeInfo;
pub use value::{AnyValue, AnyValueRef};

pub type AnyPool = crate::pool::Pool<Any>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(AnyArguments<'q>);
impl_executor_for_pool_connection!(Any, AnyConnection, AnyRow);
impl_executor_for_transaction!(Any, AnyRow);
