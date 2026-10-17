//! Criterion benchmarks: scalar reference vs dispatched (SIMD) kernels.
//! Every number in `docs/kernels.md` and `benchmarks/README.md` comes from
//! running this file. Reproduce with: `cargo bench -p kestrel-kernels`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use kestrel_kernels::{dot, hamming, l2_sq, scalar};

/// Deterministic xorshift64 PRNG (same as the crate's tests).
struct XorShift(u64);

impl XorShift {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn next_f32(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as f32 / (1u64 << 23) as f32;
        v - 1.0
    }
}

const DIMS: &[usize] = &[128, 384, 768, 1536];

fn bench_dot(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot");
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    for &dim in DIMS {
        let a: Vec<f32> = (0..dim).map(|_| rng.next_f32()).collect();
        let b: Vec<f32> = (0..dim).map(|_| rng.next_f32()).collect();
        group.throughput(Throughput::Elements(dim as u64));
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| scalar::dot(black_box(&a), black_box(&b)));
        });
        group.bench_with_input(BenchmarkId::new("dispatched", dim), &dim, |bencher, _| {
            bencher.iter(|| dot(black_box(&a), black_box(&b)));
        });
    }
    group.finish();
}

fn bench_l2_sq(c: &mut Criterion) {
    let mut group = c.benchmark_group("l2_sq");
    let mut rng = XorShift(0x1234_5678_9ABC_DEF1);
    for &dim in DIMS {
        let a: Vec<f32> = (0..dim).map(|_| rng.next_f32()).collect();
        let b: Vec<f32> = (0..dim).map(|_| rng.next_f32()).collect();
        group.throughput(Throughput::Elements(dim as u64));
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| scalar::l2_sq(black_box(&a), black_box(&b)));
        });
        group.bench_with_input(BenchmarkId::new("dispatched", dim), &dim, |bencher, _| {
            bencher.iter(|| l2_sq(black_box(&a), black_box(&b)));
        });
    }
    group.finish();
}

fn bench_hamming(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamming");
    let mut rng = XorShift(0xFEED_FACE_1234_5678);
    for &words in &[16usize, 96] {
        // words * 64 = bit-vector width (1024 / 6144 bits)
        let a: Vec<u64> = (0..words).map(|_| rng.next_u64()).collect();
        let b: Vec<u64> = (0..words).map(|_| rng.next_u64()).collect();
        group.throughput(Throughput::Elements((words * 64) as u64));
        group.bench_with_input(
            BenchmarkId::new("popcnt", words * 64),
            &words,
            |bencher, _| {
                bencher.iter(|| hamming(black_box(&a), black_box(&b)));
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50).measurement_time(std::time::Duration::from_secs(3));
    targets = bench_dot, bench_l2_sq, bench_hamming
}
criterion_main!(benches);
