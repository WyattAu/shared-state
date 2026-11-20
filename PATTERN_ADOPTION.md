# Pattern adoption

Declared per the pattern canon adoption format; every claim is verified against code in this repo.

| Pattern | Status | Evidence |
|---|---|---|
| T36 | partial | src/loom_tests.rs:4 (atomic disciplines checked under all bounded interleavings) |
| T33 | partial | src/ttl.rs:65 (take_fresh consume-once primitive for one-time token stores) |
| T11 | n/a | atomics and locks only; no task tree or supervision surface |
