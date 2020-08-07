use fnv::FnvBuildHasher;
use lru_cache::LruCache;

/// An LRU cache with string keys.
///
/// Differs from [`LruCache`] by making the removal process explicit to allow a caller to
/// clean up resources.
///
/// Intended to serve as a statement cache and a metadata cache for SQL connections.
///
#[derive(Debug)]
pub struct StringCache<T> {
    inner: LruCache<String, T, FnvBuildHasher>,
}

impl<T> StringCache<T> {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: LruCache::with_hasher(capacity, Default::default()),
        }
    }

    /// Returns a mutable reference to the value corresponding to the given key
    /// in the cache, if any.
    pub fn get_mut(&mut self, k: &str) -> Option<&mut T> {
        self.inner.get_mut(k)
    }

    /// Inserts a new item in the cache, returning the least recently used
    /// value if the cache is full, or if inserting with an existing key,
    /// the replaced existing value.
    pub fn insert(&mut self, k: &str, v: T) -> Option<T> {
        let mut lru_item = None;

        if self.capacity() == self.len() && !self.contains_key(k) {
            lru_item = self.remove_lru();
        } else if self.contains_key(k) {
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
    pub fn remove_lru(&mut self) -> Option<T> {
        self.inner.remove_lru().map(|(_, v)| v)
    }

    /// Clear all cached statements from the cache.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// True if cache has a value for the given key.
    pub fn contains_key(&mut self, k: &str) -> bool {
        self.inner.contains_key(k)
    }

    /// Returns the maximum number of statements the cache can hold.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns true if the cache capacity is more than 0.
    pub fn is_enabled(&self) -> bool {
        self.capacity() > 0
    }
}
