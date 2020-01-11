use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

// TODO: figure out a cache eviction strategy
// we currently naively cache all prepared statements which could live-leak memory
// on both the client and server if the user is synthesizing queries that are different each time

// We put an upper bound on this by setting a default max connection lifetime in pool::Options but
// that's only a band-aid

/// Per-connection prepared statement cache.
pub struct StatementCache<Id> {
    statements: HashMap<String, Id>,
    columns: HashMap<Id, Arc<HashMap<Box<str>, usize>>>,
}

impl<Id> StatementCache<Id>
where
    Id: Eq + Hash,
{
    pub fn new() -> Self {
        StatementCache {
            statements: HashMap::with_capacity(10),
            columns: HashMap::with_capacity(10),
        }
    }

    #[allow(unused)]
    pub fn has_columns(&self, id: Id) -> bool {
        self.columns.contains_key(&id)
    }

    pub fn get(&self, query: &str) -> Option<&Id> {
        self.statements.get(query)
    }

    // It is a logical error to call this without first calling [put_columns]
    pub fn get_columns(&self, id: Id) -> Arc<HashMap<Box<str>, usize>> {
        Arc::clone(&self.columns[&id])
    }

    pub fn put(&mut self, query: String, id: Id) {
        self.statements.insert(query, id);
    }

    pub fn put_columns(&mut self, id: Id, columns: HashMap<Box<str>, usize>) {
        self.columns.insert(id, Arc::new(columns));
    }
}
