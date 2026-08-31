//! Atomic counter utilities.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A thread-safe atomic counter.
pub struct SharedCounter {
    inner: AtomicU64,
}

impl Default for SharedCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedCounter {
    /// Create a new counter at zero.
    pub fn new() -> Self {
        Self {
            inner: AtomicU64::new(0),
        }
    }

    /// Create a counter with an initial value.
    pub fn with_value(value: u64) -> Self {
        Self {
            inner: AtomicU64::new(value),
        }
    }

    /// Increment by 1 and return the new value.
    pub fn increment(&self) -> u64 {
        self.inner.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Increment by `n` and return the new value.
    pub fn increment_by(&self, n: u64) -> u64 {
        self.inner.fetch_add(n, Ordering::Relaxed) + n
    }

    /// Decrement by 1 and return the new value.
    pub fn decrement(&self) -> u64 {
        self.inner.fetch_sub(1, Ordering::Relaxed) - 1
    }

    /// Get the current value.
    pub fn get(&self) -> u64 {
        self.inner.load(Ordering::Relaxed)
    }

    /// Reset to zero and return the previous value.
    pub fn reset(&self) -> u64 {
        self.inner.swap(0, Ordering::Relaxed)
    }
}

impl SharedCounter {
    /// Wrap in Arc for shared access across tasks.
    pub fn into_shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_decrement() {
        let c = SharedCounter::new();
        assert_eq!(c.increment(), 1);
        assert_eq!(c.increment(), 2);
        assert_eq!(c.decrement(), 1);
        assert_eq!(c.get(), 1);
    }

    #[test]
    fn increment_by() {
        let c = SharedCounter::new();
        assert_eq!(c.increment_by(10), 10);
        assert_eq!(c.increment_by(5), 15);
    }

    #[test]
    fn reset() {
        let c = SharedCounter::with_value(42);
        assert_eq!(c.reset(), 42);
        assert_eq!(c.get(), 0);
    }

    #[tokio::test]
    async fn concurrent_increment() {
        let counter = SharedCounter::new().into_shared();
        let mut handles = vec![];
        for _ in 0..100 {
            let c = counter.clone();
            handles.push(tokio::spawn(async move {
                c.increment();
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(counter.get(), 100);
    }
}
