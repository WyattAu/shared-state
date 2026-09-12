# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

## [0.1.1] - 2026-09-12

### Added

- `TtlCache::take_fresh` — single-use consume primitive: removes an entry
  only if it has not expired, dropping expired entries as absent. Built
  for one-time-token stores (first consumer: oauth-toolkit's CSRF store,
  replacing a hand-rolled TTL map).
- `tests/config_matrix.rs` — behavior-observable test per public knob
  (`set_ready`, `set_not_ready`, `with_value`, TTL), per the estate
  config-matrix standard.

## [0.1.0] - 2026-08-31

### Added
- Concurrent state utilities — readiness gates, TTL caches, shared counters.
- Published to crates.io (2026-08-31).
