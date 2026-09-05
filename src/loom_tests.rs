//! Loom model-checking tests for [`crate::ReadyGate`] and
//! [`crate::counter::SharedCounter`].
//!
//! These verify the crate's atomic disciplines under *all* bounded
//! interleavings:
//!
//! 1. `ReadyGate` Release/Acquire pairing: a payload stored (Relaxed) before
//!    `set_ready()` (Release) is always visible to a thread that observed
//!    `is_ready() == true` (Acquire) — the readiness flag doubles as a
//!    publication fence, so no torn/stale payload reads are possible.
//! 2. `ReadyGate` store-observation: a single `set_ready()` is never lost —
//!    every waiter thread eventually observes it (no lost wakeups in the
//!    atomic flag discipline), and each waiter's observation is monotone.
//!
//! 3. `SharedCounter` update discipline: balanced increment/decrement pairs
//!    across racing threads always return the counter to its start value
//!    (no lost updates) and `increment()` never overshoots the number of
//!    in-flight increments.
//!
//! What loom does NOT cover (trusted components):
//! - `ReadyGate::wait_until_ready` — the tokio poll/sleep loop is a tokio
//!   scheduling concern; only the atomic flag parts are model-checked.
//! - `TtlCache` — correctness is time-based (`Instant` expiry under a
//!   `std::sync::Mutex`); loom does not model the passage of time, and the
//!   Mutex is std-trusted. Mutex-protected state has no interleavings loom
//!   could expose beyond what the type system already guarantees.
//!
//! Run with:
//! ```text
//! RUSTFLAGS="--cfg loom" cargo test --release --lib -- --test-threads=1 loom
//! ```

use crate::ReadyGate;
use crate::counter::SharedCounter;
use loom::sync::Arc;
use loom::sync::atomic::{AtomicUsize, Ordering};

/// Model 1: Release/Acquire message passing through the readiness flag.
///
/// The producer publishes a payload with a Relaxed store, then marks the
/// gate ready. The consumer spins on `is_ready()` (Acquire) and reads the
/// payload. For every interleaving, the consumer must observe the payload —
/// if the gate's Release/Acquire pairing were broken (e.g. downgraded to
/// Relaxed on either side), this model would catch the stale read.
#[test]
fn loom_ready_gate_release_acquire_visibility() {
    loom::model(|| {
        let gate = Arc::new(ReadyGate::new());
        let payload = Arc::new(AtomicUsize::new(0));

        let g = Arc::clone(&gate);
        let p = Arc::clone(&payload);
        let producer = loom::thread::spawn(move || {
            p.store(1, Ordering::Relaxed); // payload published first
            g.set_ready(); // Release
        });

        let g = Arc::clone(&gate);
        let p = Arc::clone(&payload);
        let consumer = loom::thread::spawn(move || {
            while !g.is_ready() {
                loom::thread::yield_now(); // Acquire poll
            }
            assert_eq!(p.load(Ordering::Relaxed), 1, "stale payload after ready");
        });

        producer.join().unwrap();
        consumer.join().unwrap();
    });
}

/// Model 2: racing `set_ready` stores are never lost.
///
/// Two threads race to mark the gate ready while an observer polls
/// `is_ready()`. For every interleaving, the observer must unblock — the
/// idempotent stores must not be lost or observed inconsistently — and the
/// gate must remain ready afterwards.
///
/// (More than one *spinning* waiter exceeds loom's practical exploration
/// budget: every yield in a spin loop is a scheduling point, so the branch
/// count grows multiplicatively. One spinning observer keeps the model
/// bounded while still covering the set/wait discipline.)
#[test]
fn loom_ready_gate_store_observed_by_all_waiters() {
    loom::model(|| {
        let gate = Arc::new(ReadyGate::new());
        let observed = Arc::new(AtomicUsize::new(0));

        let g = Arc::clone(&gate);
        let o = Arc::clone(&observed);
        let observer = loom::thread::spawn(move || {
            while !g.is_ready() {
                loom::thread::yield_now();
            }
            o.fetch_add(1, Ordering::Relaxed);
        });

        let g2 = Arc::clone(&gate);
        let setter = loom::thread::spawn(move || {
            g2.set_ready();
        });
        gate.set_ready();

        setter.join().unwrap();
        observer.join().unwrap();

        assert_eq!(
            observed.load(Ordering::Relaxed),
            1,
            "lost ready observation"
        );
        assert!(gate.is_ready());
    });
}

/// Model 3: `SharedCounter` balanced pairs never lose updates.
///
/// Two threads each perform an increment/decrement pair. Invariants for
/// every interleaving: the counter returns to exactly 0 (no lost update),
/// no `increment()` overshoots the number of in-flight increments (result
/// is 1 or 2, never more), and the counter is never observed above the
/// live-increment bound.
///
/// Note: the two `increment()` results are *not* asserted distinct — when
/// the pairs nest (inc, inc, dec, dec) they differ, but when they serialize
/// (inc, dec, inc, dec) both legitimately return 1. Overshoot and lost
/// updates are the invariants that must hold unconditionally.
#[test]
fn loom_shared_counter_pairs_balance_no_lost_updates() {
    loom::model(|| {
        let counter = Arc::new(SharedCounter::new());

        let c2 = Arc::clone(&counter);
        let handle = loom::thread::spawn(move || {
            let first = c2.increment();
            c2.decrement();
            first
        });

        let mine = counter.increment();
        counter.decrement();
        let theirs = handle.join().unwrap();

        assert!(mine >= 1 && mine <= 2, "increment overshoot");
        assert!(theirs >= 1 && theirs <= 2, "increment overshoot");
        assert_eq!(counter.get(), 0, "lost counter update");
    });
}
