use crate::any::{
    AnyArgumentBuffer, AnyArguments, AnyConnection, AnyRow, AnyTransactionManager, AnyTypeInfo,
    AnyValue, AnyValueRef,
};
use crate::database::{Database, HasArguments, HasStatementCache, HasValueRef};

/// Opaque database driver. Capable of being used in place of any SQLx database driver. The actual
/// driver used will be selected at runtime, from the connection uri.
#[derive(Debug)]
pub struct Any;

impl Database for Any {
    type Connection = AnyConnection;

    type TransactionManager = AnyTransactionManager;

    type Row = AnyRow;

    type TypeInfo = AnyTypeInfo;

    type Value = AnyValue;
}

impl<'r> HasValueRef<'r> for Any {
    type Database = Any;

    type ValueRef = AnyValueRef<'r>;
}

impl<'q> HasArguments<'q> for Any {
    type Database = Any;

    type Arguments = AnyArguments<'q>;

    type ArgumentBuffer = AnyArgumentBuffer<'q>;
}

// This _may_ be true, depending on the selected database
impl HasStatementCache for Any {}
