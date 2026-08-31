# shared-state

Concurrent state utilities for Rust services.

Provides [`ReadyGate`] for readiness probing, [`TtlCache`] for time-expiring caches, [`SharedCounter`] for atomic counters, and type aliases for common concurrent patterns.

## Usage

```toml
[dependencies]
shared-state = "0.1"
```

## Features

- **`ReadyGate`** — Thread-safe readiness gate for Kubernetes probes
- **`TtlCache`** — Concurrent cache with per-entry TTL eviction
- **`SharedCounter`** — Atomic counter with increment/decrement/reset
- **`SharedMap<K,V>`** — Type alias for `Arc<RwLock<HashMap<K,V>>>`
- **`SharedCache<K,V>`** — Type alias for `Arc<DashMap<K,V>>`

## License

MIT OR Apache-2.0
