//! Readiness gate for service health probing.
//!
//! The atomic type is swapped for loom's equivalent under `cfg(loom)`
//! (see `loom_tests`) so the Release/Acquire discipline is model-checked.

#[cfg(loom)]
use loom::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(loom))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// A thread-safe readiness gate.
///
/// Services use this to signal when they are ready to accept traffic.
/// Kubernetes liveness/readiness probes poll `is_ready()`.
pub struct ReadyGate {
    ready: AtomicBool,
}

impl Default for ReadyGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadyGate {
    /// Create a new gate in the NOT-ready state.
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
        }
    }

    /// Mark the service as ready.
    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::Release);
    }

    /// Mark the service as NOT ready.
    pub fn set_not_ready(&self) {
        self.ready.store(false, Ordering::Release);
    }

    /// Check if the service is ready (non-blocking).
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    /// Wait until the service is ready, polling every `interval`.
    pub async fn wait_until_ready(&self) {
        loop {
            if self.is_ready() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn starts_not_ready() {
        let gate = ReadyGate::new();
        assert!(!gate.is_ready());
    }

    #[tokio::test]
    async fn becomes_ready() {
        let gate = ReadyGate::new();
        gate.set_ready();
        assert!(gate.is_ready());
    }

    #[tokio::test]
    async fn can_become_not_ready() {
        let gate = ReadyGate::new();
        gate.set_ready();
        gate.set_not_ready();
        assert!(!gate.is_ready());
    }

    #[tokio::test]
    async fn wait_until_ready() {
        let gate = ReadyGate::new();
        let gate_clone = std::sync::Arc::new(gate);
        let g = gate_clone.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            g.set_ready();
        });
        gate_clone.wait_until_ready().await;
        assert!(gate_clone.is_ready());
    }
}
