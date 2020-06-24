use lru_cache::LruCache;

/// A cache for prepared statements. When full, the least recently used
/// statement gets removed.
#[derive(Debug)]
pub struct StatementCache {
    inner: LruCache<String, u32>,
}

impl StatementCache {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: LruCache::new(capacity),
        }
    }

    /// Returns a mutable reference to the value corresponding to the given key
    /// in the cache, if any.
    pub fn get_mut(&mut self, k: &str) -> Option<&mut u32> {
        self.inner.get_mut(k)
    }

    /// Inserts a new statement to the cache, returning the least recently used
    /// statement id if the cache is full, or if inserting with an existing key,
    /// the replaced existing statement.
    pub fn insert(&mut self, k: &str, v: u32) -> Option<u32> {
        let mut lru_item = None;

        if self.inner.capacity() == self.len() && !self.inner.contains_key(k) {
            lru_item = self.remove_lru();
        } else if self.inner.contains_key(k) {
            lru_item = self.inner.remove(k);
        }

        self.inner.insert(k.into(), v);

        lru_item
    }

    /// The number of statements in the cache.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Removes the least recently used item from the cache.
    pub fn remove_lru(&mut self) -> Option<u32> {
        self.inner.remove_lru().map(|(_, v)| v)
    }
}
