mod arguments;
mod column;
mod connection;
mod database;
mod done;
mod error;
mod options;
mod row;
mod statement;
mod transaction;
mod type_info;
mod types;
mod value;

pub use arguments::AuroraArguments;
pub use column::AuroraColumn;
pub use connection::AuroraConnection;
pub use database::Aurora;
pub use done::AuroraDone;
pub use error::AuroraDatabaseError;
pub use options::AuroraConnectOptions;
pub use row::AuroraRow;
pub use statement::AuroraStatement;
pub use transaction::AuroraTransactionManager;
pub use type_info::AuroraTypeInfo;
pub use value::{AuroraValue, AuroraValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for Aurora.
pub type AuroraPool = crate::pool::Pool<Aurora>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for Aurora.
pub type PgPoolOptions = crate::pool::PoolOptions<Aurora>;

impl_into_arguments_for_arguments!(AuroraArguments);
impl_executor_for_pool_connection!(Aurora, AuroraConnection, AuroraRow);
impl_executor_for_transaction!(Aurora, AuroraRow);
impl_map_row!(Aurora, AuroraRow);
impl_acquire!(Aurora, AuroraConnection);
impl_column_index_for_row!(AuroraRow);
impl_column_index_for_statement!(AuroraStatement);
impl_into_maybe_pool!(Aurora, AuroraConnection);
impl_encode_for_option!(Aurora);
