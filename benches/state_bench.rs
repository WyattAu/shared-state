use criterion::{criterion_group, criterion_main, Criterion};
use shared_state::ReadyGate;
use shared_state::counter::SharedCounter;
use std::time::Duration;

fn bench_ready_gate_new(c: &mut Criterion) {
    c.bench_function("ready_gate_new", |b| {
        b.iter(|| {
            let gate = ReadyGate::new();
            std::hint::black_box(gate);
        });
    });
}

fn bench_ready_gate_is_ready(c: &mut Criterion) {
    let gate = ReadyGate::new();
    gate.set_ready();

    c.bench_function("ready_gate_is_ready", |b| {
        b.iter(|| {
            let ready = gate.is_ready();
            std::hint::black_box(ready);
        });
    });
}

fn bench_ready_gate_set_and_check(c: &mut Criterion) {
    c.bench_function("ready_gate_set_and_check", |b| {
        b.iter(|| {
            let gate = ReadyGate::new();
            gate.set_ready();
            let ready = gate.is_ready();
            std::hint::black_box(ready);
        });
    });
}

fn bench_ttl_cache_insert(c: &mut Criterion) {
    c.bench_function("ttl_cache_insert", |b| {
        b.iter(|| {
            let cache = shared_state::ttl::TtlCache::new(Duration::from_secs(60));
            for i in 0..100 {
                cache.insert(format!("key_{}", i), i);
            }
        });
    });
}

fn bench_ttl_cache_get(c: &mut Criterion) {
    c.bench_function("ttl_cache_get", |b| {
        b.iter(|| {
            let cache = shared_state::ttl::TtlCache::new(Duration::from_secs(60));
            for i in 0..100 {
                cache.insert(format!("key_{}", i), i);
            }
            for i in 0..100 {
                let _val = cache.get(&format!("key_{}", i));
            }
        });
    });
}

fn bench_ttl_cache_get_miss(c: &mut Criterion) {
    c.bench_function("ttl_cache_get_miss", |b| {
        b.iter(|| {
            let cache: shared_state::ttl::TtlCache<&str, i32> =
                shared_state::ttl::TtlCache::new(Duration::from_secs(60));
            for _i in 0..100 {
                let _val = cache.get(&"nonexistent");
            }
        });
    });
}

fn bench_shared_counter_new(c: &mut Criterion) {
    c.bench_function("shared_counter_new", |b| {
        b.iter(|| {
            let counter = SharedCounter::new();
            std::hint::black_box(counter);
        });
    });
}

fn bench_shared_counter_increment(c: &mut Criterion) {
    c.bench_function("shared_counter_increment", |b| {
        b.iter(|| {
            let counter = SharedCounter::new();
            for _ in 0..100 {
                counter.increment();
            }
        });
    });
}

fn bench_shared_counter_increment_by(c: &mut Criterion) {
    c.bench_function("shared_counter_increment_by", |b| {
        b.iter(|| {
            let counter = SharedCounter::new();
            counter.increment_by(100);
        });
    });
}

criterion_group!(
    benches,
    bench_ready_gate_new,
    bench_ready_gate_is_ready,
    bench_ready_gate_set_and_check,
    bench_ttl_cache_insert,
    bench_ttl_cache_get,
    bench_ttl_cache_get_miss,
    bench_shared_counter_new,
    bench_shared_counter_increment,
    bench_shared_counter_increment_by,
);
criterion_main!(benches);
