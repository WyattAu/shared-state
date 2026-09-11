# Requirements — shared-state

Numbered, testable requirements. Every requirement maps to at least one named
test; every security-relevant test cites at least one requirement. Threat
IDs reference `THREAT-MODEL.md`.

Scope note: `shared-state` provides concurrent state utilities —
`ReadyGate` (single-shot readiness broadcast), `SharedCounter` (atomic
shared counter), `TtlCache` (expiry map), and `shared_map`/`shared_cache`
constructors with ergonomic type aliases.

## Functional

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-SS-001 | `ReadyGate` starts not-ready; `set_ready` makes it ready for all observers; `set_not_ready` re-arms it | MUST |
| REQ-SS-002 | `SharedCounter` supports `increment`, `decrement`, `increment_by`, `get`, and `reset` | MUST |
| REQ-SS-003 | `TtlCache::insert`/`get` store and fetch values; `get` returns `None` for expired entries | MUST |
| REQ-SS-004 | `TtlCache::cleanup` removes expired entries; `remove` deletes a live entry; `reset` clears the cache | MUST |
| REQ-SS-005 | `len`/`is_empty` report cache occupancy; overwriting a key replaces the value | SHOULD |
| REQ-SS-006 | `shared_map`/`shared_cache`/`into_shared` produce `Arc`-shared handles | SHOULD |

## Security

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-SS-100 | All primitives are data-race-free: the atomic/lock disciplines are exhaustively model-checked under loom (T1) | MUST |
| REQ-SS-101 | Concurrent counter mutations never lose updates: paired increments and decrements balance exactly under all interleavings (T2) | MUST |
| REQ-SS-102 | Readiness visibility follows Release/Acquire semantics: a waiter observing readiness sees every write made before `set_ready` (T3) | MUST |
| REQ-SS-103 | Expired entries are never served: `get` after TTL returns `None` (T4) | MUST |

## Robustness

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-SS-200 | Cache reclamation paths (`remove`, `reset`, `cleanup`) keep occupancy accounting correct (T5) | MUST |
| REQ-SS-201 | Overwrite semantics are insert-replaces-value with correct size accounting (T6) | SHOULD |

## Traceability Matrix

| Requirement | Test (fn, file) | Property class |
|-------------|-----------------|----------------|
| REQ-SS-001 | `ready_gate_starts_not_ready`, `ready_gate_set_ready_makes_ready`, `ready_gate_toggle` (`tests/proptest.rs`); `loom_ready_gate_store_observed_by_all_waiters` (`src/loom_tests.rs`) | unit/property/loom |
| REQ-SS-002 | `increment_decrement`, `increment_by` (`src/counter.rs` tests) | unit |
| REQ-SS-003 | `insert_and_get`, `expired_entry_returns_none` (`src/cache.rs` tests) | unit |
| REQ-SS-004 | `cleanup_removes_expired`, `remove_entry`, `reset`, `ttl_cache_remove` (`tests/proptest.rs`) | unit/property |
| REQ-SS-005 | `len_and_is_empty`, `ttl_cache_len_after_inserts`, `ttl_cache_overwrite` (`tests/proptest.rs`) | unit/property |
| REQ-SS-100 | `loom_ready_gate_release_acquire_visibility`, `loom_ready_gate_store_observed_by_all_waiters`, `loom_shared_counter_pairs_balance_no_lost_updates` (`src/loom_tests.rs`, `loom` feature) | loom |
| REQ-SS-101 | `loom_shared_counter_pairs_balance_no_lost_updates`, `increment_decrement` | loom/unit |
| REQ-SS-102 | `loom_ready_gate_release_acquire_visibility` | loom |
| REQ-SS-103 | `expired_entry_returns_none`, `cleanup_removes_expired` | unit |
| REQ-SS-200 | `remove_entry`, `reset`, `len_and_is_empty` | unit/property |
| REQ-SS-201 | `ttl_cache_overwrite`, `ttl_cache_len_after_inserts` | property |

## Test Count

- 18 `#[test]` functions in the unit suite plus 7 property tests
  (`tests/proptest.rs`); 3 loom models (`loom` feature).
- All-features suite passes with 0 failures; no-default-features suite passes.
