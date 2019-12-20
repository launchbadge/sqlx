use std::collections::hash_map::{HashMap, Entry};
use std::cmp::Ordering;
use futures_core::Future;

// TODO: figure out a cache eviction strategy
// we currently naively cache all prepared statements which could live-leak memory
// on both the client and server if the user is synthesizing queries that are different each time

// We put an upper bound on this by setting a default max connection lifetime in pool::Options but
// that's only a band-aid

/// Per-connection prepared statement cache.
pub struct StatementCache<Id> {
    statements: HashMap<String, Id>
}

impl<Id> StatementCache<Id> {
    pub fn new() -> Self {
        StatementCache {
            statements: HashMap::with_capacity(10),
        }
    }

    #[cfg(feature = "mysql")]
    pub async fn get_or_compute<'a, E, Fut>(&'a mut self, query: &str, compute: impl FnOnce() -> Fut)
                                            -> Result<&'a Id, E>
    where
        Fut: Future<Output = Result<Id, E>>
    {
        match self.statements.entry(query.to_string()) {
            Entry::Occupied(occupied) => Ok(occupied.into_mut()),
            Entry::Vacant(vacant) => {
                Ok(vacant.insert(compute().await?))
            }
        }
    }

    // for Postgres so it can return the synthetic statement name instead of formatting twice
    #[cfg(feature = "postgres")]
    pub async fn map_or_compute<R, E, Fut>(&mut self, query: &str, map: impl FnOnce(&Id) -> R, compute: impl FnOnce() -> Fut)
        -> Result<R, E>
    where
        Fut: Future<Output = Result<(Id, R), E>> {

        match self.statements.entry(query.to_string()) {
            Entry::Occupied(occupied) => Ok(map(occupied.get())),
            Entry::Vacant(vacant) => {
                let (id, ret) = compute().await?;
                vacant.insert(id);
                Ok(ret)
            }
        }
    }
}
