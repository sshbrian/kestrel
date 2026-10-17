//! Kestrel core engine — first runnable slice.
//!
//! This is deliberately the smallest honest engine: an in-memory **exact**
//! (brute-force) top-k search over the SIMD kernels in `kestrel-kernels`.
//! Exact search has recall 1.0 by construction, which makes it the ground
//! truth the future ANN indexes (HNSW, IVF-PQ) will be measured against —
//! and at kernel speed it is already the right tool for small-to-medium
//! corpora and filtered candidate sets.
//!
//! No `unsafe` here: all of it lives behind the safe wrappers in
//! `kestrel-kernels`.

#![forbid(unsafe_code)]

use kestrel_kernels::{cosine, dot, l2_sq};

/// Distance/similarity metric for an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    /// Squared L2 distance. Lower is closer.
    L2Sq,
    /// Cosine similarity in `[-1, 1]`. Higher is closer.
    Cosine,
    /// Inner product. Higher is closer.
    Dot,
}

impl Metric {
    fn score(&self, query: &[f32], candidate: &[f32]) -> f32 {
        match self {
            Metric::L2Sq => l2_sq(query, candidate),
            Metric::Cosine => cosine(query, candidate),
            Metric::Dot => dot(query, candidate),
        }
    }

    /// Sort key: ascending order of this value is best-first for the metric.
    fn sort_key(&self, score: f32) -> f32 {
        match self {
            Metric::L2Sq => score,
            Metric::Cosine | Metric::Dot => -score,
        }
    }
}

/// In-memory exact-search index over fixed-dimension `f32` vectors.
///
/// Vectors are stored row-major in one flat buffer (cache-friendly scans,
/// no per-vector allocation). Ids are assigned sequentially from 0.
#[derive(Debug)]
pub struct FlatIndex {
    dim: usize,
    metric: Metric,
    data: Vec<f32>,
}

impl FlatIndex {
    /// Create an empty index for `dim`-dimensional vectors.
    pub fn new(dim: usize, metric: Metric) -> Self {
        assert!(dim > 0, "dimension must be positive");
        Self {
            dim,
            metric,
            data: Vec::new(),
        }
    }

    /// Number of stored vectors.
    pub fn len(&self) -> usize {
        self.data.len() / self.dim
    }

    /// True if the index holds no vectors.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Vector dimension this index accepts.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Insert a vector, returning its id.
    ///
    /// # Panics
    /// Panics if `vector.len() != self.dim()`.
    pub fn insert(&mut self, vector: &[f32]) -> u32 {
        assert_eq!(vector.len(), self.dim, "dimension mismatch");
        let id = self.len() as u32;
        self.data.extend_from_slice(vector);
        id
    }

    /// The stored vector for `id`, or `None` if out of range.
    pub fn get(&self, id: u32) -> Option<&[f32]> {
        let start = id as usize * self.dim;
        self.data.get(start..start + self.dim)
    }

    /// Exact top-`k` search: returns `(id, score)` pairs best-first.
    /// Returns fewer than `k` results if the index holds fewer vectors.
    /// Scores follow the metric's direction (see [`Metric`]).
    ///
    /// # Panics
    /// Panics if `query.len() != self.dim()`.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        assert_eq!(query.len(), self.dim, "dimension mismatch");
        let n = self.len();
        if n == 0 || k == 0 {
            return Vec::new();
        }
        let mut scored: Vec<(u32, f32)> = Vec::with_capacity(n);
        for id in 0..n {
            let candidate = &self.data[id * self.dim..(id + 1) * self.dim];
            scored.push((id as u32, self.metric.score(query, candidate)));
        }
        let key = |s: f32| self.metric.sort_key(s);
        scored.sort_by(|a, b| {
            key(a.1)
                .partial_cmp(&key(b.1))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        scored.truncate(k.min(n));
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kestrel_kernels::scalar;

    #[test]
    fn l2_ordering_known() {
        let mut idx = FlatIndex::new(2, Metric::L2Sq);
        idx.insert(&[10.0, 10.0]); // id 0, far
        idx.insert(&[1.0, 1.0]); // id 1, near
        idx.insert(&[5.0, 5.0]); // id 2, mid
        let hits = idx.search(&[0.0, 0.0], 3);
        let ids: Vec<u32> = hits.iter().map(|h| h.0).collect();
        assert_eq!(ids, vec![1, 2, 0]);
        assert!((hits[0].1 - 2.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_ordering_known() {
        let mut idx = FlatIndex::new(2, Metric::Cosine);
        idx.insert(&[-1.0, 0.0]); // opposite
        idx.insert(&[5.0, 0.0]); // same direction, larger magnitude
        idx.insert(&[0.0, 3.0]); // orthogonal
        let hits = idx.search(&[1.0, 0.0], 3);
        let ids: Vec<u32> = hits.iter().map(|h| h.0).collect();
        assert_eq!(ids, vec![1, 2, 0]);
        assert!((hits[0].1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn edge_cases() {
        let idx = FlatIndex::new(4, Metric::Dot);
        assert!(idx.is_empty());
        assert!(idx.search(&[0.0; 4], 10).is_empty());

        let mut idx = FlatIndex::new(4, Metric::Dot);
        idx.insert(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(idx.len(), 1);
        assert_eq!(idx.search(&[1.0; 4], 100).len(), 1); // k > len
        assert!(idx.search(&[1.0; 4], 0).is_empty()); // k = 0
        assert_eq!(idx.get(0), Some([1.0, 2.0, 3.0, 4.0].as_slice()));
        assert_eq!(idx.get(7), None);
    }

    #[test]
    #[should_panic(expected = "dimension mismatch")]
    fn insert_panics_on_wrong_dim() {
        let mut idx = FlatIndex::new(3, Metric::L2Sq);
        idx.insert(&[1.0, 2.0]);
    }

    /// Ground-truth check: top-k from the (possibly SIMD) index scan must
    /// match an independent brute-force computation using only the scalar
    /// reference kernels.
    #[test]
    fn recall_vs_scalar_ground_truth() {
        struct XorShift(u64);
        impl XorShift {
            fn next(&mut self) -> f32 {
                let mut x = self.0;
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                self.0 = x;
                ((x >> 40) as f32 / (1u64 << 23) as f32) - 1.0
            }
        }
        let dim = 128;
        let n = 500;
        let mut rng = XorShift(0xABCD_EF01_2345_6789);
        let mut idx = FlatIndex::new(dim, Metric::L2Sq);
        let mut stored = Vec::new();
        for _ in 0..n {
            let v: Vec<f32> = (0..dim).map(|_| rng.next()).collect();
            idx.insert(&v);
            stored.push(v);
        }
        let query: Vec<f32> = (0..dim).map(|_| rng.next()).collect();

        let hits = idx.search(&query, 10);
        let mut truth: Vec<(u32, f32)> = stored
            .iter()
            .enumerate()
            .map(|(i, v)| (i as u32, scalar::l2_sq(&query, v)))
            .collect();
        truth.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(a.0.cmp(&b.0)));
        truth.truncate(10);

        let hit_ids: Vec<u32> = hits.iter().map(|h| h.0).collect();
        let truth_ids: Vec<u32> = truth.iter().map(|h| h.0).collect();
        assert_eq!(hit_ids, truth_ids, "top-10 ids must match scalar truth");
    }
}
