//! Config-knob behavior matrix for shared-state.
//!
//! Every public config knob must OBSERVABLY change behavior: the table
//! below pairs a default with a configured value and asserts the observable
//! output/state differs. A knob that cannot change behavior is a bug (see
//! breaker's sliding_window_size incident).
//!
//! Time note: `TtlCache` reads `std::time::Instant` directly (no injected
//! clock), so expiry knobs need a small real wait; kept in the low
//! milliseconds like the existing `src/ttl.rs` tests.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use shared_state::counter::SharedCounter;
use shared_state::ready::ReadyGate;
use shared_state::ttl::TtlCache;

// --- ReadyGate::set_ready / set_not_ready -----------------------------------
//
// Default state is NOT-ready; each setter flips the observable gate.

#[test]
fn knob_set_ready_flips_default_not_ready() {
    let gate = ReadyGate::new();
    assert!(!gate.is_ready(), "default construction must be not-ready");
    gate.set_ready();
    assert!(gate.is_ready(), "set_ready must flip the gate");
}

#[test]
fn knob_set_not_ready_flips_back() {
    let gate = ReadyGate::new();
    gate.set_ready();
    assert!(gate.is_ready());
    gate.set_not_ready();
    assert!(!gate.is_ready(), "set_not_ready must flip the gate back");
}

#[tokio::test]
async fn ready_gate_state_is_visible_across_threads() {
    let gate = Arc::new(ReadyGate::new());
    let g = gate.clone();
    let before = tokio::spawn(async move { g.is_ready() }).await.unwrap();
    assert!(!before, "default state visible from another task");
    gate.set_ready();
    let g = gate.clone();
    let after = tokio::spawn(async move { g.is_ready() }).await.unwrap();
    assert!(after, "set_ready visible from another task");
}

// --- SharedCounter::with_value ------------------------------------------------
//
// Default starts at zero; with_value starts at the configured count and
// flows into every subsequent operation.

#[test]
fn knob_with_value_changes_starting_count() {
    let default = SharedCounter::new();
    let configured = SharedCounter::with_value(41);

    assert_eq!(default.get(), 0);
    assert_ne!(default.get(), configured.get());
    assert_eq!(configured.get(), 41);
    // The initial value flows into arithmetic, not just `get`.
    assert_eq!(configured.increment(), 42);
    assert_eq!(default.increment(), 1);
}

#[test]
fn knob_with_value_flows_into_reset_prev() {
    let counter = SharedCounter::with_value(1234);
    assert_eq!(
        counter.reset(),
        1234,
        "reset must return the configured value"
    );
    assert_eq!(counter.get(), 0);
}

// --- TtlCache ttl (constructor knob) -------------------------------------------

#[test]
fn knob_ttl_governs_expiration() {
    let fresh = TtlCache::new(Duration::from_secs(60));
    let stale = TtlCache::new(Duration::from_millis(1));
    fresh.insert("k", 1);
    stale.insert("k", 1);
    std::thread::sleep(Duration::from_millis(5));

    assert_eq!(fresh.get(&"k"), Some(1), "long TTL must keep the entry");
    assert_eq!(stale.get(&"k"), None, "1ms TTL must expire the entry");

    // Same observable for the single-use consume path.
    stale.insert("k2", 2);
    std::thread::sleep(Duration::from_millis(5));
    assert_eq!(
        stale.take_fresh(&"k2"),
        None,
        "expired entries must not be consumable"
    );
}

#[test]
fn knob_default_ttl_is_300s() {
    let cache: TtlCache<&str, u8> = TtlCache::default();
    cache.insert("k", 1);
    std::thread::sleep(Duration::from_millis(2));
    assert_eq!(
        cache.get(&"k"),
        Some(1),
        "default TTL (300s) must behave differently from a 1ms TTL"
    );
}
