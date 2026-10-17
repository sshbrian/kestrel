//! Runnable demo + mini-benchmark for the exact-search slice.
//!
//! ```sh
//! cargo run --release -p kestrel-core --example demo
//! ```
//!
//! Generates a deterministic synthetic dataset (no downloads), builds a
//! `FlatIndex`, runs timed queries, and reports QPS/latency plus recall@10
//! measured against an independent scalar-kernel ground truth. Exact search
//! is recall 1.0 by construction; the check exists to prove the SIMD scan
//! path orders results identically to the scalar reference.

use std::time::Instant;

use kestrel_core::{FlatIndex, Metric};
use kestrel_kernels::{active_backend, scalar};

struct XorShift(u64);

impl XorShift {
    fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        ((x >> 40) as f32 / (1u64 << 23) as f32) - 1.0
    }
    fn vec(&mut self, n: usize) -> Vec<f32> {
        (0..n).map(|_| self.next_f32()).collect()
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
}

fn main() {
    let dim = 128usize;
    let n = 10_000usize;
    let queries = 200usize;
    let k = 10usize;

    println!("Kestrel exact-search demo");
    println!("  kernel backend : {}", active_backend());
    println!("  dataset        : {n} vectors x {dim} dims (synthetic, deterministic)");

    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    let dataset: Vec<Vec<f32>> = (0..n).map(|_| rng.vec(dim)).collect();
    let query_set: Vec<Vec<f32>> = (0..queries).map(|_| rng.vec(dim)).collect();

    let t0 = Instant::now();
    let mut index = FlatIndex::new(dim, Metric::L2Sq);
    for v in &dataset {
        index.insert(v);
    }
    let build_ms = t0.elapsed().as_secs_f64() * 1e3;
    println!(
        "  build          : {build_ms:.2} ms for {} inserts",
        index.len()
    );

    // Timed queries through the dispatched (SIMD) kernels.
    let mut latencies_us = Vec::with_capacity(queries);
    let mut first_hits = Vec::new();
    let t0 = Instant::now();
    for (i, q) in query_set.iter().enumerate() {
        let qs = Instant::now();
        let hits = index.search(q, k);
        latencies_us.push(qs.elapsed().as_secs_f64() * 1e6);
        if i == 0 {
            first_hits = hits;
        }
    }
    let total_s = t0.elapsed().as_secs_f64();
    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("  queries        : {queries} exact top-{k} searches");
    println!(
        "  throughput     : {:.0} QPS (single thread)",
        queries as f64 / total_s
    );
    println!(
        "  latency        : p50 {:.1} us | p99 {:.1} us",
        percentile(&latencies_us, 50.0),
        percentile(&latencies_us, 99.0)
    );

    // Recall@k against scalar ground truth (independent code path).
    let q = &query_set[0];
    let mut truth: Vec<(u32, f32)> = dataset
        .iter()
        .enumerate()
        .map(|(i, v)| (i as u32, scalar::l2_sq(q, v)))
        .collect();
    truth.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(a.0.cmp(&b.0)));
    truth.truncate(k);
    let hit_set: std::collections::HashSet<u32> = first_hits.iter().map(|h| h.0).collect();
    let overlap = truth.iter().filter(|t| hit_set.contains(&t.0)).count();
    println!(
        "  recall@{k}      : {:.2} vs scalar ground truth (exact search)",
        overlap as f64 / k as f64
    );

    println!("\nTop-5 for query 0 (id, squared L2):");
    for (id, score) in first_hits.iter().take(5) {
        println!("  id {id:>5}  {score:.4}");
    }
}
