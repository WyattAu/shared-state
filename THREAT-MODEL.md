# Threat Model — shared-state

Status: **v1.0** · Method: STRIDE over the public API surface
(`ReadyGate`, `SharedCounter`, `TtlCache`, `shared_map`/`shared_cache`
constructors).

Trust boundaries: (1) concurrent threads within one process sharing
the primitives, (2) time (TTL expiry of cache entries), (3) the
`loom` model-checking harness that validates the atomic disciplines.

The crate's whole purpose is safe sharing of mutable state, so the
dominant risks are **memory-visibility and update-loss bugs** — exactly
the class loom exhaustively checks. Data races would be UB; lost updates
would corrupt counters; missed readiness signals would hang or mis-order
consumers.

## Assets

| ID | Asset | Example |
|----|-------|---------|
| A1 | Race-free shared state | Unsynchronized access causing UB under optimization |
| A2 | Update completeness | Concurrent `increment` losing a count (classic lost-update) |
| A3 | Readiness causality | A consumer observing `is_ready() == true` before the producing writes are visible |
| A4 | Bounded cache memory | TTL cache growing without bound when entries expire but are never cleaned |

## STRIDE Analysis

| # | Threat | Category | Surface | Mitigation | Verifying test |
|---|--------|---------|---------|------------|----------------|
| T1 | Data race on shared state (UB) | Tampering | all primitives | Implemented over `Arc` + lock/atomics only; the atomic disciplines of `ReadyGate` and `SharedCounter` are exhaustively model-checked under loom across all bounded interleavings | `loom_ready_gate_release_acquire_visibility`, `loom_ready_gate_store_observed_by_all_waiters`, `loom_shared_counter_pairs_balance_no_lost_updates` (`src/loom_tests.rs`) |
| T2 | Lost updates on concurrent counter mutation | Tampering | `SharedCounter::increment`/`decrement`/`increment_by` | RMW atomics with documented ordering; loom model proves paired increments/decrements balance with no lost updates | `increment_decrement`, `increment_by`, `loom_shared_counter_pairs_balance_no_lost_updates` |
| T3 | Readiness observed before payload visibility (reordering) | Spoofing | `ReadyGate::set_ready`/`is_ready` | Release-store on set, Acquire-load on observe — the exact discipline loom validates for visibility across waiter threads | `loom_ready_gate_release_acquire_visibility`, `ready_gate_set_ready_makes_ready`, `ready_gate_starts_not_ready`, `ready_gate_toggle` (`tests/proptest.rs`) |
| T4 | Stale entries served past TTL | Tampering | `TtlCache::get` | Expiry checked on access: expired entries return `None`; `cleanup` reclaims expired slots | `expired_entry_returns_none`, `cleanup_removes_expired` |
| T5 | Unbounded cache growth | DoS | `TtlCache` | `len`/`is_empty` expose size; `remove`/`reset`/`cleanup` provide explicit reclamation paths | `len_and_is_empty`, `remove_entry`, `reset`, `ttl_cache_len_after_inserts`, `ttl_cache_remove` (`tests/proptest.rs`) |
| T6 | Map key collisions / overwrite surprises | Tampering | `shared_map` | `insert` overwrites are explicit and observable; `ttl_cache_overwrite` pins the overwrite-visibility semantics | `insert_and_get`, `ttl_cache_overwrite` (`tests/proptest.rs`) |

## OPEN RISKS (missing mitigations — not fabricated)

- **OPEN-1 — `TtlCache` expiry is lazy.** Expired entries are purged on
  access or explicit `cleanup`; there is no background reaper, so an
  insert-only workload still grows memory between cleanups.
- **OPEN-2 — no per-entry TTL.** Expiry is a cache-wide duration;
  heterogeneous TTLs need separate caches.

## Out of Scope

- Cross-process sharing (single-process only; see `shm-rings` for
  cross-process state).
- Persistence of cache contents.
- Fairness/ordering guarantees among competing waiters on `ReadyGate`.

## Residual Risks

- Time is read from the standard clock; a caller with a warped clock
  (or `Instant` weirdness under suspension) gets warped TTLs —
  documented, standard-library semantics.
