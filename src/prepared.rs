use crate::{query::QueryParameters, Backend, Error, Executor, FromSqlRow, HasSqlType, ToSql};

use futures_core::{future::BoxFuture, stream::BoxStream};
use std::marker::PhantomData;

use crate::types::{HasTypeMetadata, TypeMetadata};

use std::fmt::{self, Debug};

/// A prepared statement.
pub struct PreparedStatement<DB: Backend> {
    ///
    pub identifier: <DB as Backend>::StatementIdent,
    /// The expected type IDs of bind parameters.
    pub param_types: Vec<<DB as HasTypeMetadata>::TypeId>,
    ///
    pub columns: Vec<Column<DB>>,
}

pub struct Column<DB: Backend> {
    pub name: Option<String>,
    pub table_id: Option<<DB as Backend>::TableIdent>,
    /// The type ID of this result column.
    pub type_id: <DB as HasTypeMetadata>::TypeId,
}
