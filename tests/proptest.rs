use proptest::prelude::*;
use shared_state::{ReadyGate, ttl::TtlCache};
use std::sync::Arc;
use std::time::Duration;

proptest! {
    #[test]
    fn ready_gate_starts_not_ready(_dummy in 0..1u32) {
        let gate = ReadyGate::new();
        prop_assert!(!gate.is_ready());
    }

    #[test]
    fn ready_gate_set_ready_makes_ready(_dummy in 0..1u32) {
        let gate = ReadyGate::new();
        gate.set_ready();
        prop_assert!(gate.is_ready());
    }

    #[test]
    fn ready_gate_toggle(n in 0..100u32) {
        let gate = Arc::new(ReadyGate::new());
        for _ in 0..n {
            if gate.is_ready() {
                gate.set_not_ready();
            } else {
                gate.set_ready();
            }
        }
        // After n toggles from false, result should match parity of n
        prop_assert_eq!(gate.is_ready(), n % 2 == 1);
    }

    #[test]
    fn ttl_cache_insert_and_get(key in "[a-z]{1,10}", value in 0..1000i32) {
        let cache = TtlCache::new(Duration::from_secs(60));
        cache.insert(key.clone(), value);
        prop_assert_eq!(cache.get(&key), Some(value));
    }

    #[test]
    fn ttl_cache_remove(key in "[a-z]{1,10}", value in 0..1000i32) {
        let cache = TtlCache::new(Duration::from_secs(60));
        cache.insert(key.clone(), value);
        let removed = cache.remove(&key);
        prop_assert_eq!(removed, Some(value));
        prop_assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn ttl_cache_len_after_inserts(n in 0..50usize) {
        let cache = TtlCache::new(Duration::from_secs(60));
        for i in 0..n {
            cache.insert(format!("k{i}"), i);
        }
        prop_assert_eq!(cache.len(), n);
        prop_assert_eq!(cache.is_empty(), n == 0);
    }

    #[test]
    fn ttl_cache_overwrite(key in "[a-z]{1,10}", v1 in 0..1000i32, v2 in 0..1000i32) {
        let cache = TtlCache::new(Duration::from_secs(60));
        cache.insert(key.clone(), v1);
        cache.insert(key.clone(), v2);
        prop_assert_eq!(cache.get(&key), Some(v2));
        prop_assert_eq!(cache.len(), 1);
    }
}
