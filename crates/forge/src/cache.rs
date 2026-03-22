//! LRU template source cache.
//!
//! Caches compiled template source strings so repeated renders avoid redundant
//! disk reads and preprocessing.  The cache is thread-safe and can be shared
//! across threads via `Arc<TemplateCache>`.
//!
//! # Example
//!
//! ```rust
//! use tsx_forge::cache::TemplateCache;
//!
//! let cache = TemplateCache::new(128);
//! cache.put("greeting.forge", "Hello {{ name }}!");
//! assert_eq!(cache.get("greeting.forge"), Some("Hello {{ name }}!".to_string()));
//! cache.invalidate("greeting.forge");
//! assert!(cache.get("greeting.forge").is_none());
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ---------------------------------------------------------------------------
// Inner LRU
// ---------------------------------------------------------------------------

struct Lru {
    capacity: usize,
    /// Keys ordered most-recently-used first.
    order: Vec<String>,
    store: HashMap<String, String>,
}

impl Lru {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            capacity,
            order: Vec::with_capacity(capacity),
            store: HashMap::with_capacity(capacity),
        }
    }

    fn get(&mut self, key: &str) -> Option<&str> {
        if !self.store.contains_key(key) {
            return None;
        }
        self.touch(key);
        self.store.get(key).map(|s| s.as_str())
    }

    fn put(&mut self, key: String, value: String) {
        if self.store.contains_key(&key) {
            self.touch(&key);
            self.store.insert(key, value);
            return;
        }
        if self.order.len() >= self.capacity {
            if let Some(lru_key) = self.order.pop() {
                self.store.remove(&lru_key);
            }
        }
        self.order.insert(0, key.clone());
        self.store.insert(key, value);
    }

    fn invalidate(&mut self, key: &str) {
        if self.store.remove(key).is_some() {
            self.order.retain(|k| k != key);
        }
    }

    fn invalidate_pattern(&mut self, pattern: &str) {
        let matches: Vec<String> = self
            .store
            .keys()
            .filter(|k| k.contains(pattern))
            .cloned()
            .collect();
        for k in matches {
            self.invalidate(&k);
        }
    }

    fn touch(&mut self, key: &str) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.insert(0, key.to_string());
        }
    }

    fn len(&self) -> usize {
        self.store.len()
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Thread-safe LRU cache for template source strings.
#[derive(Clone)]
pub struct TemplateCache {
    inner: Arc<RwLock<Lru>>,
}

impl TemplateCache {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Lru::new(capacity))),
        }
    }

    /// Retrieve a cached template source, or `None` if not present.
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.write().ok()?.get(key).map(|s| s.to_string())
    }

    /// Insert or update a cache entry.
    pub fn put(&self, key: impl Into<String>, value: impl Into<String>) {
        if let Ok(mut inner) = self.inner.write() {
            inner.put(key.into(), value.into());
        }
    }

    /// Remove a specific entry.
    pub fn invalidate(&self, key: &str) {
        if let Ok(mut inner) = self.inner.write() {
            inner.invalidate(key);
        }
    }

    /// Remove all entries whose keys contain `pattern`.
    pub fn invalidate_pattern(&self, pattern: &str) {
        if let Ok(mut inner) = self.inner.write() {
            inner.invalidate_pattern(pattern);
        }
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.inner.read().map(|i| i.len()).unwrap_or(0)
    }

    /// `true` when the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_get_put() {
        let c = TemplateCache::new(4);
        c.put("a", "content-a");
        c.put("b", "content-b");
        assert_eq!(c.get("a"), Some("content-a".to_string()));
        assert_eq!(c.get("b"), Some("content-b".to_string()));
        assert_eq!(c.get("c"), None);
    }

    #[test]
    fn evicts_lru_at_capacity() {
        let c = TemplateCache::new(2);
        c.put("a", "a");
        c.put("b", "b");
        // Touch 'a' → 'b' is now LRU
        let _ = c.get("a");
        c.put("c", "c"); // evicts 'b'
        assert!(c.get("b").is_none(), "'b' should have been evicted");
        assert!(c.get("a").is_some());
        assert!(c.get("c").is_some());
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn invalidate_removes_entry() {
        let c = TemplateCache::new(4);
        c.put("x", "data");
        c.invalidate("x");
        assert!(c.get("x").is_none());
        assert!(c.is_empty());
    }

    #[test]
    fn invalidate_pattern_removes_matching() {
        let c = TemplateCache::new(8);
        c.put("components/button", "btn");
        c.put("components/input", "inp");
        c.put("layouts/base", "base");
        c.invalidate_pattern("components/");
        assert!(c.get("components/button").is_none());
        assert!(c.get("components/input").is_none());
        assert_eq!(c.get("layouts/base"), Some("base".to_string()));
    }

    #[test]
    fn update_existing_key() {
        let c = TemplateCache::new(4);
        c.put("a", "v1");
        c.put("a", "v2");
        assert_eq!(c.get("a"), Some("v2".to_string()));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn capacity_one_always_evicts_previous() {
        let c = TemplateCache::new(1);
        c.put("a", "a");
        c.put("b", "b");
        assert!(c.get("a").is_none());
        assert_eq!(c.get("b"), Some("b".to_string()));
    }
}
