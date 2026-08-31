#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Concurrent state utilities for Rust services.
//!
//! Provides [`ReadyGate`] for readiness probing, [`TtlCache`] for time-expiring
//! caches, [`SharedCounter`] for atomic counters, and type aliases for common
//! concurrent patterns.

pub mod ready;
pub mod ttl;
pub mod counter;

/// Type alias for a shared concurrent map backed by `tokio::sync::RwLock`.
pub type SharedMap<K, V> = std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<K, V>>>;

/// Type alias for a shared concurrent cache backed by `DashMap`.
pub type SharedCache<K, V> = std::sync::Arc<dashmap::DashMap<K, V>>;

/// Create a new [`SharedMap`].
pub fn shared_map<K: Eq + std::hash::Hash, V>() -> SharedMap<K, V> {
    std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()))
}

/// Create a new [`SharedCache`].
pub fn shared_cache<K: Eq + std::hash::Hash + Clone, V: Clone>() -> SharedCache<K, V> {
    std::sync::Arc::new(dashmap::DashMap::new())
}
