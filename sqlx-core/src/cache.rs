use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::fmt::Display;
use crate::types::{HasSqlType, HasTypeMetadata};

// TODO: figure out a cache eviction strategy
// we currently naively cache all prepared statements which could live-leak memory
// on both the client and server if the user is synthesizing queries that are different each time

// We put an upper bound on this by setting a default max connection lifetime in pool::Options but
// that's only a band-aid

/// Per-connection prepared statement cache.
pub struct StatementCache<Id, TypeId> {
    statements: HashMap<String, Id>,
    columns: HashMap<Id, Arc<ColumnsData<TypeId>>>,
}

pub struct ColumnsData<TypeId> {
    names_map: HashMap<Box<str>, usize>,
    types: Box<[TypeId]>,
    _priv: (),
}

impl<Id, TypeId> StatementCache<Id, TypeId>
where
    Id: Eq + Hash,
{
    pub fn new() -> Self {
        StatementCache {
            statements: HashMap::with_capacity(10),
            columns: HashMap::with_capacity(10),
        }
    }

    pub fn has_columns(&self, id: Id) -> bool {
        self.columns.contains_key(&id)
    }

    pub fn get(&self, query: &str) -> Option<&Id> {
        self.statements.get(query)
    }

    // It is a logical error to call this without first calling [put_columns]
    pub fn get_columns(&self, id: Id) -> Arc<ColumnsData<TypeId>> {
        Arc::clone(&self.columns[&id])
    }

    pub fn put(&mut self, query: String, id: Id) {
        self.statements.insert(query, id);
    }

    pub fn put_columns(&mut self, id: Id, names_map: HashMap<Box<str>, usize>, types: impl Into<Box<[TypeId]>>) {
        self.columns.insert(id, Arc::new(ColumnsData { names_map, types: types.into(), _priv: () }));
    }
}

impl<TypeId> ColumnsData<TypeId> where TypeId: Display + Eq {
    pub fn get_index(&self, name: &str) -> crate::Result<usize> {
        Ok(
            *self.names_map
                .get(name)
                .ok_or_else(|| crate::Error::ColumnNotFound(name.into()))?
        )
    }

    pub(crate) fn check_type<DB, T>(&self, index: usize) -> crate::Result<()>
        where DB: HasSqlType<T>
    {
        let compat = <DB as HasSqlType<T>>::compatible_types();

        let received = self.types.get(index)
            .ok_or(crate::Error::ColumnIndexOutOfBounds {
                index,
                len: self.types.len(),
            })?;

        if compat.contains(received) {
            Ok(())
        } else {
            Err(crate::Error::ColumnTypeMismatch {
                index,
                expected: expected.to_string().into(),
                received: received.to_string().into(),
            })
        }
    }
}
