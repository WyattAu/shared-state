//! Time-to-live cache backed by DashMap.

use dashmap::DashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// A cache entry with expiration time.
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

/// A concurrent cache with per-entry TTL eviction.
///
/// Entries are automatically considered expired after their TTL elapses.
/// Expired entries are lazily evicted on access.
pub struct TtlCache<K, V> {
    inner: DashMap<K, CacheEntry<V>>,
    ttl: Duration,
}

impl<K: Eq + Hash + Clone, V: Clone> TtlCache<K, V> {
    /// Create a new cache with the given TTL for all entries.
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: DashMap::new(),
            ttl,
        }
    }

    /// Get a value if it exists and has not expired.
    pub fn get(&self, key: &K) -> Option<V> {
        let entry = self.inner.get(key)?;
        if entry.expires_at > Instant::now() {
            Some(entry.value.clone())
        } else {
            drop(entry);
            self.inner.remove(key);
            None
        }
    }

    /// Insert a value with the configured TTL.
    pub fn insert(&self, key: K, value: V) {
        self.inner.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    /// Remove an entry.
    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key).map(|e| e.1.value)
    }

    /// Remove an entry only if it has not expired, returning its value.
    ///
    /// Unlike [`TtlCache::remove`] (which returns the value even when
    /// expired), this is the single-use-consume primitive: an expired entry
    /// is dropped and reported as absent — exactly the semantics a CSRF
    /// state store or one-time token cache needs.
    pub fn take_fresh(&self, key: &K) -> Option<V> {
        let (_, entry) = self.inner.remove(key)?;
        if entry.expires_at > Instant::now() {
            Some(entry.value)
        } else {
            None
        }
    }

    /// Remove all expired entries.
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.inner.retain(|_, entry| entry.expires_at > now);
    }

    /// Return the number of entries (including expired ones).
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Return true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for TtlCache<K, V> {
    fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn insert_and_get() {
        let cache = TtlCache::new(Duration::from_secs(60));
        cache.insert("key1", 42);
        assert_eq!(cache.get(&"key1"), Some(42));
    }

    #[test]
    fn expired_entry_returns_none() {
        let cache = TtlCache::new(Duration::from_millis(1));
        cache.insert("key1", 42);
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn remove_entry() {
        let cache = TtlCache::new(Duration::from_secs(60));
        cache.insert("key1", 42);
        assert_eq!(cache.remove(&"key1"), Some(42));
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn take_fresh_returns_unexpired_and_consumes() {
        let cache = TtlCache::new(Duration::from_secs(60));
        cache.insert("key1", 42);
        assert_eq!(cache.take_fresh(&"key1"), Some(42));
        // Single use: gone after the first take.
        assert_eq!(cache.take_fresh(&"key1"), None);
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn take_fresh_drops_expired_entries() {
        let cache = TtlCache::new(Duration::from_millis(1));
        cache.insert("key1", 42);
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(
            cache.take_fresh(&"key1"),
            None,
            "expired entries must not be handed out by take_fresh"
        );
        assert_eq!(cache.len(), 0, "expired entry must be consumed");
    }

    #[test]
    fn cleanup_removes_expired() {
        let cache = TtlCache::new(Duration::from_millis(1));
        cache.insert("key1", 1);
        cache.insert("key2", 2);
        std::thread::sleep(Duration::from_millis(5));
        cache.cleanup();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn len_and_is_empty() {
        let cache = TtlCache::new(Duration::from_secs(60));
        assert!(cache.is_empty());
        cache.insert("key1", 42);
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
    }
}
