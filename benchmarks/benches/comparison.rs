use std::hash::Hasher;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{thread_rng, RngCore};

fn fnv(data: &[u8]) -> u64 {
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(data);
    hasher.finish()
}

fn all_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("32bit");
    let mut data = Vec::new();
    for size in [2, 4, 8, 16, 32, 64, 96, 128, 256, 1024] {
        let mut rng = thread_rng();
        data.resize(size, 0);
        rng.fill_bytes(&mut data);

        group.bench_function(BenchmarkId::new("cityhasher", size), |b| {
            b.iter(|| {
                let _: u32 = cityhasher::hash(black_box(&data));
            })
        });
        #[cfg(feature = "nightly")]
        group.bench_function(BenchmarkId::new("cityhash-sys", size), |b| {
            b.iter(|| cityhash_sys::city_hash_32(black_box(&data)))
        });
        group.bench_function(BenchmarkId::new("crc32fast", size), |b| {
            b.iter(|| crc32fast::hash(black_box(&data)))
        });
    }
    drop(group);

    let mut group = c.benchmark_group("64bit");
    for size in [2, 4, 8, 16, 32, 64, 96, 128, 256, 1024] {
        let mut rng = thread_rng();
        data.resize(size, 0);
        rng.fill_bytes(&mut data);

        group.bench_function(BenchmarkId::new("cityhasher", size), |b| {
            b.iter(|| {
                let _: u64 = cityhasher::hash(black_box(&data));
            })
        });
        #[cfg(feature = "nightly")]
        group.bench_function(BenchmarkId::new("cityhash-sys", size), |b| {
            b.iter(|| cityhash_sys::city_hash_64(black_box(&data)))
        });
        group.bench_function(BenchmarkId::new("fnv", size), |b| {
            b.iter(|| fnv(black_box(&data)))
        });
    }
}

criterion_group!(benches, all_benches);
criterion_main!(benches);
